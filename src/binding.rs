use super::{MTLArgumentAccess, MTLDataType, MTLStructType, MTLTextureType, NSUInteger};
use foreign_types::{ForeignType, ForeignTypeRef};
use objc::runtime::{Object, Protocol, NO, YES};
use std::any::type_name;

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

    #[inline]
    pub unsafe fn as_subprotocol_unchecked<T>(&self) -> &T
    where
        T: ::std::ops::Deref<Target = BindingRef> + ForeignTypeRef,
    {
        T::from_ptr(self.as_ptr() as _)
    }

    pub fn as_subprotocol<T>(&self) -> Option<&T>
    where
        T: ::std::ops::Deref<Target = BindingRef> + ForeignTypeRef,
    {
        let protocol_name: &'static str =
            &type_name::<<Binding as ForeignType>::CType>()[(module_path!().len() + 2/* :: */)..];
        if let Some(protocol) = Protocol::get(protocol_name) {
            let c = unsafe { (&*(self.as_ptr() as *const Object)).class() };
            if !c.conforms_to(protocol) {
                return None;
            }
        } else {
            return None;
        }
        Some(unsafe { self.as_subprotocol_unchecked() })
    }
}

pub struct MTLBufferBinding {}

foreign_obj_type! {
    type CType = MTLBufferBinding;
    pub struct BufferBinding;
    pub struct BufferBindingRef;
    type ParentType = BindingRef;
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

pub struct MTLTextureBinding {}

foreign_obj_type! {
    type CType = MTLTextureBinding;
    pub struct TextureBinding;
    pub struct TextureBindingRef;
    type ParentType = BindingRef;
}

impl TextureBindingRef {
    pub fn array_length(&self) -> NSUInteger {
        unsafe { msg_send![self, arrayLength] }
    }

    pub fn is_depth_texture(&self) -> bool {
        unsafe {
            match msg_send![self, isDepthTexture] {
                YES => true,
                NO => false,
                _ => unreachable!(),
            }
        }
    }

    pub fn texture_data_type(&self) -> MTLDataType {
        unsafe { msg_send![self, textureDataType] }
    }

    pub fn texture_type(&self) -> MTLTextureType {
        unsafe { msg_send![self, textureType] }
    }
}