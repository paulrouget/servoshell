use cocoa::appkit::*;
use cocoa::base::*;
use cocoa::foundation::*;
use std::sync::{Once, ONCE_INIT};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc_foundation::{INSObject, NSObject};
use std::os::raw::c_void;
static INIT: Once = ONCE_INIT;

pub fn nsWindowRegister() {
    INIT.call_once(|| {
        unsafe {

            extern fn draw_rect(_this: &Object, _sel: Sel, _rect: id) {
                println!("1");

                glClearColor(0, 0, 0, 0);
                glClear(GL_COLOR_BUFFER_BIT);
                drawAnObject();
                glFlush();

            }

            let superclass = Class::get("NSOpenGLView").unwrap();
            let mut decl = ClassDecl::new("NSOpenGLView2", superclass).unwrap();

            decl.add_method(sel!(drawRect:), draw_rect as extern fn(&Object, Sel, id));

            decl.register();
        }
    });

}
