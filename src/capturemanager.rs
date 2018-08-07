// Copyright 2018 GFX developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use super::*;

use objc_foundation::{INSString, NSString};

pub enum MTLCaptureScope {}

foreign_obj_type! {
    type CType = MTLCaptureScope;
    pub struct CaptureScope;
    pub struct CaptureScopeRef;
}

impl CaptureScopeRef {
    pub fn begin_scope(&self) {
        unsafe {
            msg_send![self, beginScope];
        }
    }

    pub fn end_scope(&self) {
        unsafe {
            msg_send![self, endScope];
        }
    }

    pub fn label(&self) -> &str {
        unsafe {
            let label: &NSString = msg_send![self, label];
            label.as_str()
        }
    }
}

pub enum MTLCaptureManager {}

foreign_obj_type! {
    type CType = MTLCaptureManager;
    pub struct CaptureManager;
    pub struct CaptureManagerRef;
}

impl CaptureManager {
    pub fn shared<'a>() -> &'a CaptureManagerRef {
        unsafe {
            let class = class!(MTLCaptureManager);
            msg_send![class, sharedCaptureManager]
        }
    }
}

impl CaptureManagerRef {
    pub fn new_capture_scope_with_device(&self, device: &DeviceRef) -> &CaptureScopeRef {
        unsafe { msg_send![self, newCaptureScopeWithDevice: device] }
    }

    pub fn new_capture_scope_with_command_queue(
        &self,
        command_queue: &CommandQueueRef,
    ) -> &CaptureScopeRef {
        unsafe { msg_send![self, newCaptureScopeWithCommandQueue: command_queue] }
    }

    pub fn default_capture_scope(&self) -> Option<&CaptureScopeRef> {
        unsafe { msg_send![self, defaultCaptureScope] }
    }

    pub fn set_default_capture_scope(&self, scope: &CaptureScopeRef) {
        unsafe { msg_send![self, setDefaultCaptureScope:scope] }
    }

    pub fn start_capture_with_device(&self, device: &DeviceRef) {
        unsafe {
            msg_send![self, startCaptureWithDevice: device];
        }
    }

    pub fn start_capture_with_command_queue(&self, command_queue: &CommandQueueRef) {
        unsafe {
            msg_send![self, startCaptureWithCommandQueue: command_queue];
        }
    }

    pub fn start_capture_with_scope(&self, scope: &CaptureScopeRef) {
        unsafe {
            msg_send![self, startCaptureWithScope: scope];
        }
    }

    pub fn stop_capture(&self) {
        unsafe {
            msg_send![self, stopCapture];
        }
    }

    pub fn is_capturing(&self) -> bool {
        unsafe { msg_send![self, isCapturing] }
    }
}