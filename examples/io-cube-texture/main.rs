#![feature(maybe_uninit_uninit_array)]
use metal::*;
use std::sync::{Arc, Mutex};

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
    let tmp_dir = std::env::temp_dir().join(&asset_dir_name);
    println!(
        "Writing cube face textures {}...",
        tmp_dir.to_string_lossy()
    );
    let face_files: [String; 6] = {
        std::fs::create_dir(&tmp_dir).expect("Failed to create temp directory");
        [0, 1, 2, 3, 4, 5].map(|face_id| {
            tmp_dir
                .join(format!("temp-texture-{face_id}.lz4"))
                .to_str()
                .expect("Failed to create path to store temporary compressed data")
                .to_owned()
        })
    };

    let all_src_texture_bytes = debug_time("Write Cube Faces", || {
        // TODO: START HERE
        // TODO: START HERE
        // TODO: START HERE
        // Use std::thread::scope();
        let all_src_texture_bytes = Arc::new(Mutex::<[(Vec<u8>, u64, u64); 6]>::new([
            (vec![], 0, 0),
            (vec![], 0, 0),
            (vec![], 0, 0),
            (vec![], 0, 0),
            (vec![], 0, 0),
            (vec![], 0, 0),
        ]));
        let mut handles = vec![];
        for (face_id, face_file) in face_files.iter().enumerate() {
            let face_file = face_file.to_owned();
            let all_src_texture_bytes = Arc::clone(&all_src_texture_bytes);
            handles.push(std::thread::spawn(move || {
                let (src_texture_bytes, (img_width, img_height)) =
                    debug_time("Read PNG", || load_image_bytes_from_png(face_id));
                debug_time("MTLIO writing compressed texture", || {
                    let io = IOCompression::new(
                        &face_file,
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
                });
                all_src_texture_bytes.lock().unwrap()[face_id] =
                    (src_texture_bytes, img_width, img_height);
            }));
        }
        for handle in handles {
            handle.join().expect("Load image did complete");
        }
        all_src_texture_bytes
    });
    let all_src_texture_bytes = &*all_src_texture_bytes.lock().unwrap();

    // Verify all cube face textures are the same dimensions
    let mut width = 0;
    let mut height = 0;
    for &(_, img_width, img_height) in all_src_texture_bytes.iter() {
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

    debug_time("MTLIO load all compressed textures", || {
        let queue = {
            let desc = IOCommandQueueDescriptor::new();
            desc.set_max_command_buffer_count(6);
            desc.set_max_commands_in_flight(1);
            device
                .new_io_command_queue(&desc)
                .expect("Failed to create IO Command Queue")
        };
        let mut command_bufs = vec![];

        // TODO: Try using only one command once MacOS 13/Xcode 14 is stable
        // - There seems to be a data corruption issue when writing all the faces of a cube texture
        //   at once (single queue, single command buffer, single commit).
        // - Make sure to adjust/remove above `set_max_command_buffer_count` and
        //   `set_max_commands_in_flight` configuration.
        for (face_id, face_file) in face_files.iter().enumerate() {
            let command_buffer = queue.new_command_buffer_with_unretained_references();
            let handle = device
                .new_io_handle(
                    URL::new_with_string(&format!("file:///{face_file}")),
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
                    depth: 1,
                },
                width * BYTES_PER_PIXELS,
                height * width * BYTES_PER_PIXELS,
                MTLOrigin { x: 0, y: 0, z: 0 },
                &handle,
                0,
            );
            command_buffer.commit();
            command_bufs.push(command_buffer);
        }
        for (face_id, command_buffer) in command_bufs.into_iter().enumerate() {
            command_buffer.wait_until_completed();
            assert_eq!(
                command_buffer.status(),
                MTLIOStatus::complete,
                "Failed to load texture for face {face_id}"
            );
        }
    });
    debug_time("Verifying cube texture contents", || {
        for (face_id, (src_texture_bytes, _, _)) in all_src_texture_bytes.iter().enumerate() {
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
            if &texture_bytes != src_texture_bytes {
                println!(
                    "Cube texture face #{} contents are incorrect: {:?} {:?}",
                    face_id,
                    &texture_bytes[0..4],
                    &src_texture_bytes[0..4],
                );
            }
        }
    });
}

fn debug_time<T>(label: &'static str, f: impl FnOnce() -> T) -> T {
    #[cfg(debug_assertions)]
    {
        use std::time::Instant;
        const MICROS_PER_MILLI: u128 = 1000;
        let now = Instant::now();
        let r = f();
        let elapsed = now.elapsed();
        let elapsed_micro = elapsed.as_micros();
        let (elapsed_display, unit) = if elapsed_micro > MICROS_PER_MILLI {
            (elapsed_micro / MICROS_PER_MILLI, "ms")
        } else {
            (elapsed_micro, "μ")
        };
        println!("[{label:<40}] {:>6} {}", elapsed_display, unit);
        return r;
    }
    #[cfg(not(debug_assertions))]
    {
        return f();
    }
}
