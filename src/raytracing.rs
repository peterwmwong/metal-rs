use crate::{ArrayRef, BufferRef, MTLAttributeFormat, MTLIndexType, NSUInteger, ResourceRef};

pub enum MTLAccelerationStructureGeometryDescriptor {}
foreign_obj_type! {
    type CType = MTLAccelerationStructureGeometryDescriptor;
    pub struct AccelerationStructureGeometryDescriptor;
    pub struct AccelerationStructureGeometryDescriptorRef;
}

impl AccelerationStructureGeometryDescriptorRef {
    pub fn set_opaque(&self, is_opaque: bool) {
        unsafe { msg_send![self, setOpaque: is_opaque] }
    }
    pub fn set_label(&self, label: &str) {
        unsafe {
            let nslabel = crate::nsstring_from_str(label);
            let () = msg_send![self, setLabel: nslabel];
        }
    }
}

pub enum MTLAccelerationStructureTriangleGeometryDescriptor {}
foreign_obj_type! {
    type CType = MTLAccelerationStructureTriangleGeometryDescriptor;
    pub struct AccelerationStructureTriangleGeometryDescriptor;
    pub struct AccelerationStructureTriangleGeometryDescriptorRef;
    type ParentType = AccelerationStructureGeometryDescriptorRef;
}

impl AccelerationStructureTriangleGeometryDescriptor {
    pub fn descriptor() -> Self {
        unsafe {
            let class = class!(MTLAccelerationStructureTriangleGeometryDescriptor);
            msg_send![class, descriptor]
        }
    }
}

impl AccelerationStructureTriangleGeometryDescriptorRef {
    pub fn set_vertex_buffer(&self, buffer: Option<&BufferRef>) {
        unsafe { msg_send![self, setVertexBuffer: buffer] }
    }
    pub fn set_vertex_buffer_offset(&self, offset: NSUInteger) {
        unsafe { msg_send![self, setVertexBufferOffset: offset] }
    }
    pub fn set_vertex_format(&self, format: MTLAttributeFormat) {
        unsafe { msg_send![self, setVertexFormat: format] }
    }
    pub fn set_vertex_stride(&self, stride: NSUInteger) {
        unsafe { msg_send![self, setVertexStride: stride] }
    }

    pub fn set_index_buffer(&self, buffer: Option<&BufferRef>) {
        unsafe { msg_send![self, setIndexBuffer: buffer] }
    }
    pub fn set_index_buffer_offset(&self, offset: NSUInteger) {
        unsafe { msg_send![self, setIndexBufferOffset: offset] }
    }
    pub fn set_index_type(&self, index_type: MTLIndexType) {
        unsafe { msg_send![self, setIndexType: index_type] }
    }

    pub fn set_triangle_count(&self, count: NSUInteger) {
        unsafe { msg_send![self, setTriangleCount: count] }
    }

    // TODO: Implement
    // vertexFormat: MTLAttributeFormat;
    // transformationMatrixBuffer: MTLBuffer;
    // transformationMatrixBufferOffset: NSUInteger;
}

pub enum MTLAccelerationStructureDescriptor {}

foreign_obj_type! {
    type CType = MTLAccelerationStructureDescriptor;
    pub struct AccelerationStructureDescriptor;
    pub struct AccelerationStructureDescriptorRef;
}

pub enum MTLPrimitiveAccelerationStructureDescriptor {}

foreign_obj_type! {
    type CType = MTLPrimitiveAccelerationStructureDescriptor;
    pub struct PrimitiveAccelerationStructureDescriptor;
    pub struct PrimitiveAccelerationStructureDescriptorRef;
    type ParentType = AccelerationStructureDescriptorRef;
}

impl PrimitiveAccelerationStructureDescriptor {
    pub fn descriptor() -> Self {
        unsafe {
            let class = class!(MTLPrimitiveAccelerationStructureDescriptor);
            msg_send![class, descriptor]
        }
    }
}

impl PrimitiveAccelerationStructureDescriptorRef {
    pub fn set_geometry_descriptors(
        &self,
        geometries: &ArrayRef<AccelerationStructureGeometryDescriptor>,
    ) {
        unsafe { msg_send![self, setGeometryDescriptors: geometries] }
    }
}

pub enum MTLAccelerationStructure {}
foreign_obj_type! {
    type CType = MTLAccelerationStructure;
    pub struct AccelerationStructure;
    pub struct AccelerationStructureRef;
    type ParentType = ResourceRef;
}

pub enum MTLInstanceAccelerationStructureDescriptor {}
foreign_obj_type! {
    type CType = MTLInstanceAccelerationStructureDescriptor;
    pub struct InstanceAccelerationStructureDescriptor;
    pub struct InstanceAccelerationStructureDescriptorRef;
    type ParentType = AccelerationStructureDescriptorRef;
}

impl InstanceAccelerationStructureDescriptor {
    pub fn descriptor() -> Self {
        unsafe {
            let class = class!(MTLInstanceAccelerationStructureDescriptor);
            msg_send![class, descriptor]
        }
    }
}
impl InstanceAccelerationStructureDescriptorRef {
    pub fn set_instanced_acceleration_structures(
        &self,
        acceleration_structure: &ArrayRef<AccelerationStructure>,
    ) {
        unsafe {
            msg_send![
                self,
                setInstancedAccelerationStructures: acceleration_structure
            ]
        }
    }

    pub fn set_instance_count(&self, instance_count: NSUInteger) {
        unsafe { msg_send![self, setInstanceCount: instance_count] }
    }

    pub fn set_instance_descriptor_buffer(&self, buffer: Option<&BufferRef>) {
        unsafe { msg_send![self, setInstanceDescriptorBuffer: buffer] }
    }
}

bitflags! {
    pub struct MTLAccelerationStructureInstanceOptions: u32 {
        const None = 0;
        const DisableTriangleCulling = (1 << 0);
        const TriangleFrontFacingWindingCounterClockwise = (1 << 1);
        const Opaque = (1 << 2);
        const NonOpaque = (1 << 3);
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct MTLPackedFloat3(pub f32, pub f32, pub f32);

#[repr(C)]
#[derive(Copy, Clone, PartialEq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct MTLPackedFloat4x3 {
    pub columns: [MTLPackedFloat3; 4],
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct MTLAccelerationStructureInstanceDescriptor {
    pub transformation_matrix: MTLPackedFloat4x3,
    pub options: MTLAccelerationStructureInstanceOptions,
    pub mask: u32,
    pub intersection_function_table_offset: u32,
    pub acceleration_structure_index: u32,
}
