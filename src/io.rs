use crate::BufferRef;

use super::{MTLOrigin, MTLSize, NSUInteger, TextureRef};
use std::ffi::{c_void, CString};
use std::os::raw::c_char;

pub enum MTLIOCommandBuffer {}

foreign_obj_type! {
    type CType = MTLIOCommandBuffer;
    pub struct IOCommandBuffer;
    pub struct IOCommandBufferRef;
}

impl IOCommandBufferRef {
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

    pub fn enqueue(&self) {
        unsafe { msg_send![self, enqueue] }
    }

    pub fn commit(&self) {
        unsafe { msg_send![self, commit] }
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

    // TODO: Implement!
    // maxCommandBufferCount: Int
    // priority: MTLIOPriority
    // type: MTLIOCommandQueueType
    // maxCommandsInFlight: Int
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
