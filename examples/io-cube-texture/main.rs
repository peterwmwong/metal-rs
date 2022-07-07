use metal::*;

const PIXEL_FORMAT: MTLPixelFormat = MTLPixelFormat::RGBA8Unorm;
const BYTES_PER_PIXELS: u64 = 4; // RGBA
const COMPRESSION_METHOD: MTLIOCompressionMethod = MTLIOCompressionMethod::lz4;
const FACE_IMAGES: [&'static [u8]; 6] = [
    include_bytes!("cubemap_posx.png"),
    include_bytes!("cubemap_negx.png"),
    include_bytes!("cubemap_posy.png"),
    include_bytes!("cubemap_negy.png"),
    include_bytes!("cubemap_posz.png"),
    include_bytes!("cubemap_negz.png"),
];

fn load_image_bytes_from_png(face_id: usize) -> (Vec<u8>, (u64, u64)) {
    assert!(face_id < 6);
    let img = FACE_IMAGES[face_id];
    let decoder = png::Decoder::new(img);
    let (info, mut reader) = decoder
        .read_info()
        .expect("Failed to decode PNG information");
    let mut buf = vec![0; info.buffer_size()];
    reader
        .next_frame(&mut buf)
        .expect("Failed to load image data into buffer");
    (buf, (info.width as _, info.height as _))
}

fn main() {
    let asset_dir_name = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("Failed to get epoch time (for temp asset directory)")
        .as_millis()
        .to_string();
    let tmp_dir = std::env::temp_dir().join(asset_dir_name);
    std::fs::create_dir(&tmp_dir).expect("Failed to create temp directory");

    let tmp_files = [0, 1, 2, 3, 4, 5].map(|face_id| {
        tmp_dir
            .join(format!("temp-texture-{face_id}.lz4"))
            .to_str()
            .expect("Failed to create path to store temporary compressed data")
            .to_owned()
    });

    let mut width = 0;
    let mut height = 0;
    let mut all_src_texture_bytes = vec![];
    for (face_id, tmp_file) in tmp_files.iter().enumerate() {
        let (src_texture_bytes, (img_width, img_height)) = load_image_bytes_from_png(face_id);
        assert!(
            (width == 0 && img_width > 0) || (width == img_width),
            "Width is invalid, must match other cube face textures"
        );
        assert!(
            (height == 0 && img_height > 0) || (height == img_height),
            "Height is invalid, must match other cube face textures"
        );
        width = img_width;
        height = img_height;

        println!("Using MTLIO to write an image into a compressed file ({tmp_file:?})...");
        let io = IOCompression::new(
            tmp_file,
            COMPRESSION_METHOD,
            IOCompression::default_chunk_size(),
        );
        io.append(
            src_texture_bytes.as_ptr() as _,
            src_texture_bytes.len() as _,
        );
        let io_flush_result = io.flush();
        assert_eq!(
            io_flush_result,
            MTLIOCompressionStatus::complete,
            "Failed to write compressed file"
        );
        println!("... write completed!");
        all_src_texture_bytes.push(src_texture_bytes);
    }

    let device = Device::system_default().expect("No device found");
    let texture = {
        let desc = TextureDescriptor::new();
        desc.set_texture_type(MTLTextureType::Cube);
        desc.set_width(width);
        desc.set_height(height);
        desc.set_pixel_format(PIXEL_FORMAT);
        desc.set_resource_options(MTLResourceOptions::StorageModeShared);
        device.new_texture(&desc)
    };
    {
        let width = texture.width();
        let height = texture.height();
        let depth = texture.depth();
        let total_bytes = height * width * BYTES_PER_PIXELS;

        let queue = {
            let desc = IOCommandQueueDescriptor::new();
            desc.set_max_commands_in_flight(6);
            desc.set_max_command_buffer_count(1);
            device
                .new_io_command_queue(&desc)
                .expect("Failed to create IO Command Queue")
        };
        let command_buffer = queue.new_command_buffer();
        for (face_id, tmp_file) in tmp_files.iter().enumerate() {
            println!(
                "Using MTLIO to read a compressed file ({:?}) into texture...",
                &tmp_file
            );
            let handle = device
                .new_io_handle(
                    URL::new_with_string(&format!("file:///{tmp_file}")),
                    COMPRESSION_METHOD,
                )
                .expect("Failed to get IO file handle");

            command_buffer.load_texture(
                &texture,
                face_id as _,
                0,
                MTLSize {
                    width,
                    height,
                    depth,
                },
                width * BYTES_PER_PIXELS,
                total_bytes,
                MTLOrigin { x: 0, y: 0, z: 0 },
                &handle,
                0,
            );
        }
        command_buffer.commit();
        command_buffer.wait_until_completed();
        assert_eq!(
            command_buffer.status(),
            MTLIOStatus::complete,
            "Failed to load texture"
        );
        println!("... read completed!");
    }
    {
        println!("Verifying texture contents match originally written image...");
        for (face_id, src_texture_bytes) in all_src_texture_bytes.into_iter().enumerate() {
            let mut texture_bytes = vec![0_u8; src_texture_bytes.len()];
            println!("Verifiying face {face_id}");
            texture.get_bytes_in_slice(
                texture_bytes.as_mut_ptr() as _,
                width * BYTES_PER_PIXELS,
                height * width * BYTES_PER_PIXELS,
                MTLRegion {
                    origin: MTLOrigin { x: 0, y: 0, z: 0 },
                    size: MTLSize {
                        width,
                        height,
                        depth: 1,
                    },
                },
                0,
                face_id as _,
            );
            if &texture_bytes != &src_texture_bytes {
                println!(
                    "Cube texture face #{} contents are incorrect: {:?} {:?}",
                    face_id,
                    &texture_bytes[0..4],
                    &src_texture_bytes[0..4],
                );
            }
        }
        println!("... contents verified!");
    }
}
