use crate::BufferRef;

use super::{MTLOrigin, MTLSize, NSUInteger, TextureRef};
use std::ffi::{c_void, CString};
use std::os::raw::c_char;

#[allow(non_camel_case_types)]
#[repr(u64)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MTLIOStatus {
    pending = 0,
    cancelled = 1,
    error = 2,
    complete = 3,
}

pub enum MTLIOCommandBuffer {}

foreign_obj_type! {
    type CType = MTLIOCommandBuffer;
    pub struct IOCommandBuffer;
    pub struct IOCommandBufferRef;
}

impl IOCommandBufferRef {
    pub fn commit(&self) {
        unsafe { msg_send![self, commit] }
    }

    pub fn enqueue(&self) {
        unsafe { msg_send![self, enqueue] }
    }

    pub fn label(&self) -> &str {
        unsafe {
            let label = msg_send![self, label];
            crate::nsstring_as_str(label)
        }
    }
    pub fn set_label(&self, label: &str) {
        unsafe {
            let nslabel = crate::nsstring_from_str(label);
            let () = msg_send![self, setLabel: nslabel];
        }
    }

    pub fn load_buffer(
        &self,
        buffer: &BufferRef,
        offset: NSUInteger,
        size: NSUInteger,
        source_handle: &IOFileHandleRef,
        source_handle_offset: NSUInteger,
    ) {
        unsafe {
            msg_send![self,
                loadBuffer:buffer
                offset:offset
                size:size
                sourceHandle:source_handle
                sourceHandleOffset:source_handle_offset
            ]
        }
    }

    pub fn load_texture(
        &self,
        texture: &TextureRef,
        slice: NSUInteger,
        level: NSUInteger,
        size: MTLSize,
        source_bytes_per_row: NSUInteger,
        source_bytes_per_image: NSUInteger,
        destination_origin: MTLOrigin,
        source_handle: &IOFileHandleRef,
        source_handle_offset: NSUInteger,
    ) {
        unsafe {
            msg_send![self,
                loadTexture:texture
                slice:slice
                level:level
                size:size
                sourceBytesPerRow:source_bytes_per_row
                sourceBytesPerImage:source_bytes_per_image
                destinationOrigin:destination_origin
                sourceHandle:source_handle
                sourceHandleOffset:source_handle_offset
            ]
        }
    }

    pub fn status(&self) -> MTLIOStatus {
        unsafe { msg_send![self, status] }
    }

    pub fn wait_until_completed(&self) {
        unsafe { msg_send![self, waitUntilCompleted] }
    }
}

pub enum MTLIOCommandQueue {}

foreign_obj_type! {
    type CType = MTLIOCommandQueue;
    pub struct IOCommandQueue;
    pub struct IOCommandQueueRef;
}

impl IOCommandQueueRef {
    pub fn label(&self) -> &str {
        unsafe {
            let label = msg_send![self, label];
            crate::nsstring_as_str(label)
        }
    }

    pub fn set_label(&self, label: &str) {
        unsafe {
            let nslabel = crate::nsstring_from_str(label);
            let () = msg_send![self, setLabel: nslabel];
        }
    }

    pub fn new_command_buffer(&self) -> &IOCommandBufferRef {
        unsafe { msg_send![self, commandBuffer] }
    }

    pub fn new_command_buffer_with_unretained_references(&self) -> &IOCommandBufferRef {
        unsafe { msg_send![self, commandBufferWithUnretainedReferences] }
    }

    // TODO: Implement
    // enqueueBarrier()
}

pub enum MTLIOCommandQueueDescriptor {}

foreign_obj_type! {
    type CType = MTLIOCommandQueueDescriptor;
    pub struct IOCommandQueueDescriptor;
    pub struct IOCommandQueueDescriptorRef;
}

impl IOCommandQueueDescriptor {
    pub fn new() -> Self {
        unsafe {
            let class = class!(MTLIOCommandQueueDescriptor);
            msg_send![class, new]
        }
    }
}

impl IOCommandQueueDescriptorRef {
    pub fn max_command_buffer_count(&self) -> NSUInteger {
        unsafe { msg_send![self, maxCommandBufferCount] }
    }
    pub fn set_max_command_buffer_count(&self, count: NSUInteger) {
        unsafe {
            let () = msg_send![self, setMaxCommandBufferCount: count];
        }
    }

    pub fn max_commands_in_flight(&self) -> NSUInteger {
        unsafe { msg_send![self, maxCommandsInFlight] }
    }
    pub fn set_max_commands_in_flight(&self, count: NSUInteger) {
        unsafe {
            let () = msg_send![self, setMaxCommandsInFlight: count];
        }
    }

    // TODO: Implement!
    // priority: MTLIOPriority
    // type: MTLIOCommandQueueType
    // scratchBufferAllocator: MTLIOScratchBufferAllocator?
}

pub enum MTLIOFileHandle {}

foreign_obj_type! {
    type CType = MTLIOFileHandle;
    pub struct IOFileHandle;
    pub struct IOFileHandleRef;
}

impl MTLIOFileHandle {
    pub fn label(&self) -> &str {
        unsafe {
            let label = msg_send![self, label];
            crate::nsstring_as_str(label)
        }
    }

    pub fn set_label(&self, label: &str) {
        unsafe {
            let nslabel = crate::nsstring_from_str(label);
            let () = msg_send![self, setLabel: nslabel];
        }
    }
}

// Available on macOS 13.0
#[allow(non_camel_case_types)]
#[repr(u64)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MTLIOCompressionMethod {
    zlib = 0,
    lzfse = 1,
    lz4 = 2,
    lzma = 3,
    lzBitmap = 4,
}

#[allow(non_camel_case_types)]
#[repr(u64)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MTLIOCompressionStatus {
    complete = 0,
    error = 1,
}

#[allow(non_camel_case_types)]
#[link(name = "Metal", kind = "framework")]
extern "C" {
    // Available on macOS 13.0
    fn MTLIOCreateCompressionContext(
        path: *const c_char,
        compression_type: MTLIOCompressionMethod,
        chunkSize: NSUInteger,
    ) -> *const c_void;

    // Available on macOS 13.0
    fn MTLIOCompressionContextAppendData(
        context: *const c_void,
        data: *const c_void,
        size: NSUInteger,
    );

    fn MTLIOFlushAndDestroyCompressionContext(context: *const c_void) -> MTLIOCompressionStatus;
}

#[allow(non_camel_case_types)]
pub struct IOCompression {
    context: *const c_void,
}

#[allow(non_camel_case_types)]
impl IOCompression {
    // TODO: Re-implement once macOS 13/xcode 14 releases and API stabilizes
    // - Currently...
    //   - In Beta 2, NOT in Documentation: `public let kMTLIOCompressionContextDefaultChunkSize: Int`
    //   - NOT in Beta 2, In Documentation: `size_t MTLIOCompressionContextDefaultChunkSize(void);`
    //      - https://developer.apple.com/documentation/metal/4048349-mtliocompressioncontextdefaultch?language=objc
    //   - HHHHahhahhaahahhhhh.......... whatevs
    pub fn default_chunk_size() -> NSUInteger {
        65536
    }

    pub fn new(
        path: &str,
        compression_type: MTLIOCompressionMethod,
        chunk_size: NSUInteger,
    ) -> Self {
        let path = CString::new(path).expect("Could not create compatible string for path");
        Self {
            context: unsafe {
                MTLIOCreateCompressionContext(path.as_ptr() as _, compression_type, chunk_size)
            },
        }
    }

    pub fn append(&self, bytes: *const std::ffi::c_void, length: NSUInteger) {
        unsafe { MTLIOCompressionContextAppendData(self.context, bytes, length) };
    }

    pub fn flush(self) -> MTLIOCompressionStatus {
        unsafe { MTLIOFlushAndDestroyCompressionContext(self.context) }
    }
}
