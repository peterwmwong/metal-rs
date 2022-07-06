use metal::*;
use std::ops::Range;

const PIXEL_FORMAT: MTLPixelFormat = MTLPixelFormat::RGBA8Unorm;
const BYTES_PER_PIXELS: u64 = 4; // RGBA
const COMPRESSION_METHOD: MTLIOCompressionMethod = MTLIOCompressionMethod::lz4;
const FACE_RANGE: Range<u64> = 0..6;

fn load_image_bytes_from_png() -> (Vec<u8>, (u64, u64)) {
    let img = include_bytes!("./cubemap_negy.png");
    let decoder = png::Decoder::new(img.as_ref());
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
    let tmp_dir = std::env::temp_dir();
    let tmp_raw_file_path = tmp_dir.join("temp-texture.raw");
    let tmp_file_path = tmp_dir.join("temp-texture.lz4");
    let tmp_file = tmp_file_path
        .to_str()
        .expect("Failed to create path to store temporary compressed data");

    let (src_texture_bytes, (width, height)) = load_image_bytes_from_png();
    println!("Write image bytes into {tmp_raw_file_path:?}");
    std::fs::write(tmp_raw_file_path, &src_texture_bytes).expect("Failed to write raw image bytes");

    {
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
        let queue = device
            .new_io_command_queue(&IOCommandQueueDescriptor::new())
            .expect("Failed to create IO Command Queue");
        let command_buffer = queue.new_command_buffer();
        let width = texture.width();
        let height = texture.height();
        let depth = texture.depth();
        let total_bytes = height * width * BYTES_PER_PIXELS;
        for i in FACE_RANGE {
            command_buffer.load_texture(
                &texture,
                i,
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
        let mut texture_bytes = vec![0_u8; src_texture_bytes.len()];
        for i in FACE_RANGE {
            println!("Verifiying face {i}");
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
                i,
            );
            assert_eq!(&texture_bytes, &src_texture_bytes);
        }
        println!("... contents verified!");
    }
}
