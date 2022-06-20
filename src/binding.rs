use crate::{MTLDataType, MTLStructType};

use super::{MTLArgumentAccess, NSUInteger};
use objc::runtime::{NO, YES};

#[repr(i64)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MTLBindingType {
    buffer = 0,
    threadgroupMemory = 1,
    texture = 2,
    sampler = 3,
    imageblockData = 16,
    imageblock = 17,
    visibleFunctionTable = 24,
    primitiveAccelerationStructure = 25,
    instanceAccelerationStructure = 26,
    intersectionFunctionTable = 27,
    objectPayload = 34,
}

pub enum MTLBinding {}

foreign_obj_type! {
    type CType = MTLBinding;
    pub struct Binding;
    pub struct BindingRef;
}

impl BindingRef {
    pub fn access(&self) -> MTLArgumentAccess {
        unsafe { msg_send![self, access] }
    }

    pub fn index(&self) -> NSUInteger {
        unsafe { msg_send![self, index] }
    }

    pub fn is_argument(&self) -> bool {
        unsafe {
            match msg_send![self, isArgument] {
                YES => true,
                NO => false,
                _ => unreachable!(),
            }
        }
    }

    pub fn is_used(&self) -> bool {
        unsafe {
            match msg_send![self, isUsed] {
                YES => true,
                NO => false,
                _ => unreachable!(),
            }
        }
    }

    pub fn name(&self) -> &str {
        unsafe {
            let name = msg_send![self, name];
            crate::nsstring_as_str(name)
        }
    }

    pub fn type_(&self) -> MTLBindingType {
        unsafe { msg_send![self, type] }
    }

    pub fn is_active(&self) -> bool {
        unsafe {
            match msg_send![self, isActive] {
                YES => true,
                NO => false,
                _ => unreachable!(),
            }
        }
    }
}

pub struct MTLBufferBinding {}

foreign_obj_type! {
    type CType = MTLBufferBinding;
    pub struct BufferBinding;
    pub struct BufferBindingRef;
    type ParentType = MTLBinding;
}

impl BufferBindingRef {
    pub fn buffer_alignment(&self) -> NSUInteger {
        unsafe { msg_send![self, bufferAlignment] }
    }

    pub fn buffer_data_size(&self) -> NSUInteger {
        unsafe { msg_send![self, bufferDataSize] }
    }

    pub fn buffer_data_type(&self) -> MTLDataType {
        unsafe { msg_send![self, bufferDataType] }
    }

    // TODO: Implement
    // pub fn buffer_pointer_type(&self) -> Option<MTLPointerType> {
    //     unsafe { msg_send![self, bufferPointerType] }
    // }

    pub fn buffer_struct_type(&self) -> Option<MTLStructType> {
        unsafe { msg_send![self, bufferStructType] }
    }
}
