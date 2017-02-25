use cocoa::appkit::*;
use cocoa::base::*;
use cocoa::foundation::*;
use std::sync::{Once, ONCE_INIT};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc_foundation::{INSObject, NSObject};
use std::os::raw::c_void;
static INIT: Once = ONCE_INIT;

pub fn new_app_delegate() -> id {
    unsafe {
        INIT.call_once(|| {
            extern fn flush_gl_context(_this: &Object, _sel: Sel, id: id) {
                println!("flush_gl_context");
                msg_send![id, flushBuffer];
            }
            let superclass = Class::get("NSObject").unwrap();
            let mut decl = ClassDecl::new("NSMyAppDelegate", superclass).unwrap();
            decl.add_method(sel!(flushGlContext), flush_gl_context as extern fn(&Object, Sel));
            decl.register();
        });
        msg_send![Class::get("NSMyAppDelegate").unwrap(), alloc]
    }
}
