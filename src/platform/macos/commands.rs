
use cocoa::appkit::*;
use cocoa::foundation::*;
use cocoa::base::*;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use std::os::raw::c_void;
use std::sync::{Once, ONCE_INIT};
use super::utils;

static INIT: Once = ONCE_INIT;

pub fn register() {
    unsafe {
        INIT.call_once(|| {

            let superclass = Class::get("NSObjectController").unwrap();
            let mut class = ClassDecl::new("NSShellCommands", superclass).unwrap();

            class.add_method(sel!(reload:), a as extern fn(&Object, Sel, id));
            extern fn a(this: &Object, _sel: Sel, _: id) {
                println!("reload");
            }

            class.add_method(sel!(validateMenuItem:), x as extern fn(&Object, Sel, id) -> BOOL);
            extern fn x(this: &Object, _sel: Sel, _: id) -> BOOL {
                println!("validateMenuItem");
                NO
            }

            class.register();
        });
    }
}


pub struct Commands {
    nscontroller: id,
}

impl Commands {
    pub fn new(nscontroller: id) -> Commands {
        Commands {
            nscontroller: nscontroller,
        }
    }
}
