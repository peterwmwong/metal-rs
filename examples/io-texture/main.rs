use metal::*;

const PIXEL_FORMAT: MTLPixelFormat = MTLPixelFormat::RGBA8Unorm;
const BYTES_PER_PIXELS: u64 = 4; // RGBA
const COMPRESSION_METHOD: MTLIOCompressionMethod = MTLIOCompressionMethod::lz4;

fn load_image_bytes_from_png() -> (Vec<u8>, (u64, u64)) {
    let img = include_bytes!("./gfx-rs.png");
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

fn write_compressed_data(src_data: &[u8], dest_file: &str) {
    println!("Using MTLIO to write an image into a compressed file ({dest_file:?})...");
    let io = IOCompression::new(
        dest_file,
        COMPRESSION_METHOD,
        IOCompression::default_chunk_size(),
    );
    io.append(src_data.as_ptr() as _, src_data.len() as _);
    let io_flush_result = io.flush();
    assert_eq!(
        io_flush_result,
        MTLIOCompressionStatus::complete,
        "Failed to write compressed file"
    );
    println!("... write completed!");
}

fn read_compressed_data(device: &Device, texture: &TextureRef, src_file: &str) {
    println!(
        "Using MTLIO to read a compressed file ({:?}) into texture...",
        &src_file
    );
    let handle = device
        .new_io_handle(
            URL::new_with_string(&format!("file:///{src_file}")),
            COMPRESSION_METHOD,
        )
        .expect("Failed to get IO file handle");
    let queue = device
        .new_io_command_queue(&IOCommandQueueDescriptor::new())
        .expect("Failed to create IO Command Queue");
    let command_buffer = queue.new_command_buffer_with_unretained_references();
    let width = texture.width();
    let height = texture.height();
    command_buffer.load_texture(
        texture,
        0,
        0,
        MTLSize {
            width,
            height,
            depth: 0,
        },
        width * BYTES_PER_PIXELS,
        height * width * BYTES_PER_PIXELS,
        MTLOrigin { x: 0, y: 0, z: 0 },
        &handle,
        0,
    );
    command_buffer.commit();
    command_buffer.wait_until_completed();
    assert_eq!(
        command_buffer.status(),
        MTLIOStatus::complete,
        "Failed to load texture"
    );
    println!("... read completed!");
}

fn main() {
    let tmp_file_path = std::env::temp_dir().join("temp-texture.lz4");
    let tmp_file = tmp_file_path
        .to_str()
        .expect("Failed to create path to store temporary compressed data");

    let (src_texture_bytes, (width, height)) = load_image_bytes_from_png();
    write_compressed_data(&src_texture_bytes, tmp_file);

    let device = Device::system_default().expect("No device found");
    let dest_texture = TextureDescriptor::new();
    dest_texture.set_width(width);
    dest_texture.set_height(height);
    dest_texture.set_pixel_format(PIXEL_FORMAT);
    let texture = device.new_texture(&dest_texture);
    read_compressed_data(&device, &texture, tmp_file);

    println!("Verifying texture contents match originally written image...");
    let mut texture_bytes = vec![0_u8; src_texture_bytes.len()];
    texture.get_bytes(
        texture_bytes.as_mut_ptr() as _,
        width * BYTES_PER_PIXELS,
        MTLRegion {
            origin: MTLOrigin { x: 0, y: 0, z: 0 },
            size: MTLSize {
                width,
                height,
                depth: 0,
            },
        },
        0,
    );
    assert_eq!(&texture_bytes, &src_texture_bytes);
    println!("... contents verified!");
}
