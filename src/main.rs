#![feature(box_syntax)]

#[macro_use]
extern crate objc;
extern crate gleam;
extern crate cocoa;
extern crate objc_foundation;
extern crate libc;
extern crate core_foundation;
extern crate cgl;

use cocoa::appkit::*;
use cocoa::base::*;
use cocoa::foundation::*;

use cgl::{CGLEnable, kCGLCECrashOnRemovedFunctions};

use std::env::args;
use std::str::FromStr;
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use core_foundation::bundle::{CFBundleGetBundleWithIdentifier, CFBundleGetFunctionPointerForName};
use std::os::raw::c_void;

mod app;
mod servo;

use servo::{FollowLinkPolicy, Servo};

#[derive(Copy, Clone)]
pub struct DrawableGeometry {
    inner_size: (u32, u32),
    position: (i32, i32),
    hidpi_factor: f32,
}

// Used by Servo to wake up the event loop
pub struct EventLoopRiser {}

impl EventLoopRiser {
    pub fn rise(&self) {
        println!("riser");
    }
    pub fn clone(&self) -> EventLoopRiser {
        EventLoopRiser {}
    }
}


fn main() {
    let (app, glview) = app::load_nib();

    let cxt = unsafe {

        let attributes = vec![
            NSOpenGLPFADoubleBuffer as u32,
            NSOpenGLPFAClosestPolicy as u32,
            NSOpenGLPFAColorSize as u32,
            32,
            NSOpenGLPFAAlphaSize as u32,
            8,
            NSOpenGLPFADepthSize as u32,
            24,
            NSOpenGLPFAStencilSize as u32,
            8,
            NSOpenGLPFAOpenGLProfile as u32,
            NSOpenGLProfileVersion3_2Core as u32,
            0,
        ];

        let pixelformat = NSOpenGLPixelFormat::alloc(nil).initWithAttributes_(&attributes);
        let cxt: id = NSOpenGLContext::alloc(nil).initWithFormat_shareContext_(pixelformat, nil);
        // let pf = {
        //     let get_attr = |attrib: appkit::NSOpenGLPixelFormatAttribute| -> i32 {
        //         let mut value = 0;
        //         NSOpenGLPixelFormat::getValues_forAttribute_forVirtualScreen_(
        //             *pixelformat,
        //             &mut value,
        //             attrib,
        //             NSOpenGLContext::currentVirtualScreen(*cxt));

        //         value
        //     };

        //     PixelFormat {
        //         hardware_accelerated: get_attr(appkit::NSOpenGLPFAAccelerated) != 0,
        //         color_bits: (get_attr(appkit::NSOpenGLPFAColorSize) - get_attr(appkit::NSOpenGLPFAAlphaSize)) as u8,
        //         alpha_bits: get_attr(appkit::NSOpenGLPFAAlphaSize) as u8,
        //         depth_bits: get_attr(appkit::NSOpenGLPFADepthSize) as u8,
        //         stencil_bits: get_attr(appkit::NSOpenGLPFAStencilSize) as u8,
        //         stereoscopy: get_attr(appkit::NSOpenGLPFAStereo) != 0,
        //         double_buffer: get_attr(appkit::NSOpenGLPFADoubleBuffer) != 0,
        //         multisampling: if get_attr(appkit::NSOpenGLPFAMultisample) > 0 {
        //             Some(get_attr(appkit::NSOpenGLPFASamples) as u16)
        //         } else {
        //             None
        //         },
        //         srgb: true,
        //     }
        // };

        msg_send![cxt, setView:glview];
        let value = 0;
        cxt.setValues_forParameter_(&value, NSOpenGLContextParameter::NSOpenGLCPSwapInterval);
        CGLEnable(cxt.CGLContextObj() as *mut _, kCGLCECrashOnRemovedFunctions);
        cxt
    };


    gleam::gl::load_with(|addr| {
        let symbol_name: CFString = FromStr::from_str(addr).unwrap();
        let framework_name: CFString = FromStr::from_str("com.apple.opengl").unwrap();
        let framework = unsafe { CFBundleGetBundleWithIdentifier(framework_name.as_concrete_TypeRef()) };
        let symbol = unsafe { CFBundleGetFunctionPointerForName(framework, symbol_name.as_concrete_TypeRef()) };
        symbol as *const c_void
    });
    gleam::gl::clear_color(1.0, 0.0, 0.0, 1.0);
    gleam::gl::clear(gleam::gl::COLOR_BUFFER_BIT);
    gleam::gl::finish();

    let url = args().nth(1).unwrap_or("http://servo.org".to_owned());
    let servo = Servo::new(DrawableGeometry { inner_size: (200, 200), position: (0, 0), hidpi_factor: 1.0, },
                           EventLoopRiser {},
                           &url,
                           FollowLinkPolicy::FollowOriginalDomain,
                           cxt);

    unsafe {
        app.run();
    }
}
