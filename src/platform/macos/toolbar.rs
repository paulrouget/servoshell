use cocoa::appkit::*;
use cocoa::foundation::*;
use cocoa::base::*;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use super::get_state;

pub fn register() {

    let superclass = Class::get("NSSegmentedCell").unwrap();
    let mut class = ClassDecl::new("NSShellSegmentedCell", superclass).unwrap();

    extern fn draw(this: &Object, _sel: Sel, rect: NSRect, view: id) {
        unsafe {
            let dark = get_state().dark_theme;
            if dark {
                msg_send![this, drawInteriorWithFrame:rect inView:view];
            } else {
                msg_send![super(this, Class::get("NSSegmentedCell").unwrap()), drawWithFrame:rect inView:view];
            }
        }
    }

    unsafe {
        class.add_method(sel!(drawWithFrame:inView:), draw as extern fn(&Object, Sel, NSRect, id));
    }

    class.register();
}
