use metal::*;

const COMPRESSION_CHUNK_SIZE: NSUInteger = 256;
const COMPRESSION_METHOD: MTLIOCompressionMethod = MTLIOCompressionMethod::lz4;

fn write_compressed_data(src_data: &str, dest_file: &str) {
    println!("Using MTLIO to write a compressed file ({dest_file:?})...");
    let io = IOCompression::new(dest_file, COMPRESSION_METHOD, COMPRESSION_CHUNK_SIZE);
    io.append(src_data.as_ptr() as _, src_data.len() as _);
    let io_flush_result = io.flush();
    assert_eq!(
        io_flush_result,
        MTLIOCompressionStatus::complete,
        "Failed to write compressed file"
    );
    println!("... write completed!");
}

fn read_compressed_data(device: &Device, buffer: &BufferRef, src_file: &str) {
    println!(
        "Using MTLIO to read compressed file ({:?}) into buffer...",
        &src_file
    );
    let handle = device
        .new_io_handle(
            URL::new_with_string(&format!("file:///{src_file}")),
            MTLIOCompressionMethod::lz4,
        )
        .expect("Failed to get IO file handle");
    let queue = device
        .new_io_command_queue(&IOCommandQueueDescriptor::new())
        .expect("Failed to create IO Command Queue");
    let command_buffer = queue.new_command_buffer_with_unretained_references();
    command_buffer.load_buffer(&buffer, 0, buffer.length(), &handle, 0);
    command_buffer.commit();
    command_buffer.wait_until_completed();
    println!("... read completed!");
}

fn main() {
    let tmp_file_path = std::env::temp_dir().join("temp.lz4");
    let tmp_file = tmp_file_path
        .to_str()
        .expect("Failed to create path to store temporary compressed data");

    let src_data = "yolo".repeat(256);
    write_compressed_data(&src_data, tmp_file);

    let device = Device::system_default().expect("No device found");
    let buffer = device.new_buffer(src_data.len() as _, MTLResourceOptions::StorageModeShared);
    read_compressed_data(&device, &buffer, tmp_file);

    println!("Verifying contents match originally written data...");
    let contents =
        unsafe { std::slice::from_raw_parts(buffer.contents() as *const u8, src_data.len()) };
    assert_eq!(contents, src_data.as_bytes());
    println!("... contents verified!");
}
