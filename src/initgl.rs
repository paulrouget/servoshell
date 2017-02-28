extern crate core_foundation;
extern crate cgl;
extern crate gleam;

use cocoa::appkit::*;
use cocoa::base::*;
use self::cgl::{CGLEnable, kCGLCECrashOnRemovedFunctions};
use std::str::FromStr;
use self::core_foundation::base::TCFType;
use self::core_foundation::string::CFString;
use self::core_foundation::bundle::{CFBundleGetBundleWithIdentifier, CFBundleGetFunctionPointerForName};
use std::os::raw::c_void;

pub fn init(nsview: id) -> id /* NSOpenGLContext */ {
    let ctx = unsafe {
        nsview.setWantsBestResolutionOpenGLSurface_(YES);
        let attributes = vec![NSOpenGLPFADoubleBuffer as u32,
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
                              0];

        let pixelformat = NSOpenGLPixelFormat::alloc(nil).initWithAttributes_(&attributes);
        let ctx: id = NSOpenGLContext::alloc(nil).initWithFormat_shareContext_(pixelformat, nil);
        msg_send![ctx, setView: nsview];
        let value = 1;
        ctx.setValues_forParameter_(&value, NSOpenGLContextParameter::NSOpenGLCPSwapInterval);
        CGLEnable(ctx.CGLContextObj() as *mut _, kCGLCECrashOnRemovedFunctions);
        ctx
    };

    unsafe {
        msg_send![ctx, update];
        msg_send![ctx, makeCurrentContext];
    };

    gleam::gl::load_with(|addr| {
        let symbol_name: CFString = FromStr::from_str(addr).unwrap();
        let framework_name: CFString = FromStr::from_str("com.apple.opengl").unwrap();
        let framework = unsafe {
            CFBundleGetBundleWithIdentifier(framework_name.as_concrete_TypeRef())
        };
        let symbol = unsafe {
            CFBundleGetFunctionPointerForName(framework, symbol_name.as_concrete_TypeRef())
        };
        symbol as *const c_void
    });

    gleam::gl::clear_color(1.0, 0.0, 0.0, 1.0);
    gleam::gl::clear(gleam::gl::COLOR_BUFFER_BIT);
    gleam::gl::finish();

    ctx
}
