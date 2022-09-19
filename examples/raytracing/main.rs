use metal::{
    AccelerationStructureGeometryDescriptorRef, AccelerationStructureRef,
    AccelerationStructureTriangleGeometryDescriptor, Array, CompileOptions, Device, HeapDescriptor,
    InstanceAccelerationStructureDescriptor, MTLAccelerationStructureInstanceDescriptor,
    MTLAccelerationStructureInstanceOptions, MTLAttributeFormat, MTLCommandBufferStatus,
    MTLIndexType, MTLPackedFloat3, MTLPackedFloat4x3, MTLResourceOptions, MTLResourceUsage,
    MTLSize, MTLSizeAndAlign, MTLStorageMode, PrimitiveAccelerationStructureDescriptor,
};

const INTERSECTION_RESULT_HIT: u32 = 1234;
const INTERSECTION_RESULT_MISS: u32 = 0;

fn main() {
    let device = Device::system_default().expect("Failed to access Metal Device");
    let cmd_queue = device.new_command_queue();

    // ======================================
    // Build Primitive Acceleration Structure
    // ======================================
    let tri_format = MTLAttributeFormat::Float3;
    // packed_float3 has a size of 12 bytes.
    // See Section 2.2.3 "Packed Vector Types", Table 2.4 https://developer.apple.com/metal/Metal-Shading-Language-Specification.pdf
    let tri_stride = 12;
    let tri: [f32; 12] = [
        7., 7., 7., /* random pad */
        -1., -1., 1., /* 0 */
        0., 1., 1., /* 1 */
        1., -1., 1., /* 2 */
    ];
    let tri_buffer = device.new_buffer_with_data(
        (&tri as *const f32) as *const _,
        std::mem::size_of_val(&tri) as _,
        MTLResourceOptions::StorageModeShared,
    );

    let index_type = MTLIndexType::UInt16;
    let indices: [u16; 3] = [1, 2, 3];
    let indices_buffer = device.new_buffer_with_data(
        (&indices as *const u16) as *const _,
        std::mem::size_of_val(&indices) as _,
        MTLResourceOptions::StorageModeShared,
    );

    let primitive_data: [u32; 3] = [INTERSECTION_RESULT_HIT, 7, 8];
    let primitive_data_buffer = device.new_buffer_with_data(
        (&primitive_data as *const u32) as *const _,
        std::mem::size_of_val(&primitive_data) as _,
        MTLResourceOptions::StorageModeShared,
    );
    let as_geo_tri = AccelerationStructureTriangleGeometryDescriptor::descriptor();
    as_geo_tri.set_vertex_format(tri_format);
    as_geo_tri.set_vertex_buffer(Some(&tri_buffer));
    as_geo_tri.set_vertex_buffer_offset(0);
    as_geo_tri.set_vertex_stride(tri_stride);
    as_geo_tri.set_index_buffer(Some(&indices_buffer));
    as_geo_tri.set_index_buffer_offset(0);
    as_geo_tri.set_index_type(index_type);
    as_geo_tri.set_triangle_count(1);
    as_geo_tri.set_opaque(true);
    as_geo_tri.set_label("Triangle Geometry Acceleration Structure");
    as_geo_tri.set_primitive_data_buffer(Some(&primitive_data_buffer));
    as_geo_tri.set_primitive_data_buffer_offset(0);
    as_geo_tri.set_primitive_data_element_size(std::mem::size_of_val(&primitive_data) as _);
    as_geo_tri.set_primitive_data_stride(std::mem::size_of_val(&primitive_data) as _);

    let as_primitive_desc = PrimitiveAccelerationStructureDescriptor::descriptor();
    as_primitive_desc.set_geometry_descriptors(Array::from_slice(&[
        &as_geo_tri as &AccelerationStructureGeometryDescriptorRef
    ]));

    let MTLSizeAndAlign { size, align } =
        device.heap_acceleration_structure_size_and_align(&as_primitive_desc);
    let mut sizes = device.acceleration_structure_sizes_with_descriptor(&as_primitive_desc);
    sizes.acceleration_structure_size = size + align;

    let heap_with_as_primitive = {
        let desc = HeapDescriptor::new();
        desc.set_storage_mode(MTLStorageMode::Private);
        desc.set_size(sizes.acceleration_structure_size);
        device.new_heap(&desc)
    };
    let as_primitive = heap_with_as_primitive
        .new_acceleration_structure(size)
        .expect("Failed to allocate acceleration structure");

    let scratch_buffer = device.new_buffer(
        sizes.build_scratch_buffer_size,
        MTLResourceOptions::StorageModePrivate,
    );
    let cmd_buf = cmd_queue.new_command_buffer();
    let encoder = cmd_buf.new_acceleration_structure_command_encoder();
    encoder.use_heap(&heap_with_as_primitive);
    encoder.build_acceleration_structure(&as_primitive, &as_primitive_desc, &scratch_buffer, 0);
    encoder.end_encoding();
    cmd_buf.commit();
    cmd_buf.wait_until_completed();

    // =====================================
    // Build Instance Acceleration Structure
    // =====================================
    let as_instance_desc = InstanceAccelerationStructureDescriptor::descriptor();
    as_instance_desc.set_instanced_acceleration_structures(&Array::from_slice(&[
        &as_primitive as &AccelerationStructureRef
    ]));
    as_instance_desc.set_instance_count(1);

    let as_instance_descriptor_buffer = device.new_buffer_with_data(
        (&MTLAccelerationStructureInstanceDescriptor {
            // Identity Matrix (column major 4x3)
            transformation_matrix: MTLPackedFloat4x3 {
                columns: [
                    MTLPackedFloat3(1., 0., 0.),
                    MTLPackedFloat3(0., 1., 0.),
                    MTLPackedFloat3(0., 0., 1.),
                    MTLPackedFloat3(0., 0., 0.),
                ],
            },
            options: MTLAccelerationStructureInstanceOptions::None,
            mask: 0xFF,
            intersection_function_table_offset: 0,
            acceleration_structure_index: 0,
        } as *const MTLAccelerationStructureInstanceDescriptor) as *const _,
        std::mem::size_of::<MTLAccelerationStructureInstanceDescriptor>() as _,
        MTLResourceOptions::StorageModeShared,
    );
    as_instance_desc.set_instance_descriptor_buffer(Some(&as_instance_descriptor_buffer));
    let cmd_buf = cmd_queue.new_command_buffer();
    let sizes = device.acceleration_structure_sizes_with_descriptor(&as_instance_desc);
    let scratch_buffer = device.new_buffer(
        sizes.build_scratch_buffer_size,
        MTLResourceOptions::StorageModePrivate,
    );
    let as_instance = device
        .new_acceleration_structure(sizes.acceleration_structure_size)
        .expect("Failed to allocate instance acceleration structure");

    let encoder = cmd_buf.new_acceleration_structure_command_encoder();
    encoder.use_heap(&heap_with_as_primitive);
    encoder.build_acceleration_structure(&as_instance, &as_instance_desc, &scratch_buffer, 0);
    encoder.end_encoding();
    cmd_buf.commit();
    cmd_buf.wait_until_completed();

    // ===========================
    // Performing Ray Intersection
    // ===========================
    let lib = device
        .new_library_with_source(
            &format!(
                r#"
#include <metal_stdlib>

using namespace metal;
using raytracing::instance_acceleration_structure;

[[kernel]]
void main_kernel(
             instance_acceleration_structure   accelerationStructure [[buffer(0)]],
    constant float3                          & direction             [[buffer(1)]],
    device   uint                            * output                [[buffer(2)]]
) {{
    raytracing::ray r;
    r.origin       = float3(0);
    r.direction    = normalize(direction);
    r.min_distance = 0.1;
    r.max_distance = FLT_MAX;

    raytracing::intersector<raytracing::instancing> inter;
    inter.assume_geometry_type(raytracing::geometry_type::triangle);
    auto intersection = inter.intersect(r, accelerationStructure, 0xFF);
    if (intersection.type == raytracing::intersection_type::triangle) {{
        *output = *((const device unsigned int*) intersection.primitive_data);
    }} else {{
        *output = {INTERSECTION_RESULT_MISS};
    }}
}}
    "#
            ),
            &CompileOptions::new(),
        )
        .expect("Failed to compile shader");
    let main_kernel_fn = lib
        .get_function("main_kernel", None)
        .expect("Failed to get kernel function");
    let output_buffer = device.new_buffer(
        std::mem::size_of::<u32>() as _,
        MTLResourceOptions::StorageModeShared,
    );
    let pipeline = device
        .new_compute_pipeline_state_with_function(&main_kernel_fn)
        .expect("Failed to create compute pipeline");

    let assert_ray_intersection = |is_hit: bool, direction: [f32; 3]| {
        let cmd_buf = cmd_queue.new_command_buffer();
        let encoder = cmd_buf.new_compute_command_encoder();
        encoder.set_acceleration_structure(Some(&as_instance), 0);
        encoder.set_bytes(
            1,
            std::mem::size_of_val(&direction) as _,
            (&direction as *const f32) as *const _,
        );
        encoder.set_buffer(2, Some(&output_buffer), 0);
        encoder.set_compute_pipeline_state(&pipeline);
        encoder.use_heap(&heap_with_as_primitive);
        encoder.dispatch_threads(
            MTLSize {
                width: 1,
                height: 1,
                depth: 1,
            },
            MTLSize {
                width: 1,
                height: 1,
                depth: 1,
            },
        );
        encoder.end_encoding();
        cmd_buf.commit();
        cmd_buf.wait_until_completed();
        let output_value = unsafe { &*(output_buffer.contents() as *const u32) };
        let expected_output_value = if is_hit {
            INTERSECTION_RESULT_HIT
        } else {
            INTERSECTION_RESULT_MISS
        };
        assert_eq!(
            &expected_output_value, output_value,
            "Unexpected output value for direction {direction:?}"
        );
    };

    assert_ray_intersection(true, [0_f32, 0., 1.]);
    assert_ray_intersection(false, [1_f32, 0., 0.]);
    assert_ray_intersection(false, [0_f32, 1., 0.]);
    assert_ray_intersection(false, [1_f32, 0., 1.]);

    // Verify refitting acceleration structure: move the instance right (translate x by 1.0)
    {
        unsafe {
            let a: &mut MTLAccelerationStructureInstanceDescriptor =
                &mut *(as_instance_descriptor_buffer.contents() as *mut _);
            a.transformation_matrix = MTLPackedFloat4x3 {
                columns: [
                    MTLPackedFloat3(1., 0., 0.),
                    MTLPackedFloat3(0., 1., 0.),
                    MTLPackedFloat3(0., 0., 1.),
                    MTLPackedFloat3(1., 0., 0.),
                ],
            };
        };
        let cmd_buf = cmd_queue.new_command_buffer();
        let encoder = cmd_buf.new_acceleration_structure_command_encoder();
        encoder.use_resource(&as_instance_descriptor_buffer, MTLResourceUsage::Read);
        encoder.refit(&as_instance, &as_instance_desc, None, &scratch_buffer, 0);
        encoder.end_encoding();
        cmd_buf.commit();
        cmd_buf.wait_until_completed();
        assert_eq!(cmd_buf.status(), MTLCommandBufferStatus::Completed);
    }

    assert_ray_intersection(false, [0_f32, 0., 1.]);
    assert_ray_intersection(false, [1_f32, 0., 0.]);
    assert_ray_intersection(false, [0_f32, 1., 0.]);
    assert_ray_intersection(true, [1_f32, 0., 1.]);

    println!("Completed Successfully!");
}
