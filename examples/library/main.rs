// Copyright 2016 metal-rs developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate metal_rs as metal;

use metal::*;

const PROGRAM: &'static str = "";

fn main() {
    let device = create_system_default_device();

    let options = MTLCompileOptions::new();
    let library = device.new_library_with_source(PROGRAM, options);
}