use cocoa::appkit::*;
use cocoa::base::*;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};


pub fn register() {
    let superclass = Class::get("NSToolbarItem").unwrap();
    let mut class = ClassDecl::new("NSValidableToolbarItem", superclass).unwrap();

    extern fn validate(this: &mut Object, _sel: Sel) {
        // From: https://developer.apple.com/library/content/documentation/Cocoa/Conceptual/Toolbars/Tasks/ValidatingTBItems.html#//apple_ref/doc/uid/20000753
        // Validation for view items is not automatic because a view item can be of
        // unknown complexity. To implement validation for a view item, you must
        // subclass NSToolbarItem and override validate (because NSToolbarItem’s
        // implementation of validate does nothing for view items). In your override
        // method, do the validation specific to the behavior of the view item and then
        // enable or disable whatever you want in the contents of the view accordingly.
        // If the view is an NSControl you can call setEnabled:, which will in turn
        // call setEnabled: on the control.
        unsafe {
            let action: id = msg_send![this, action];
            let target: id = msg_send![this, target];
            let responder: id = if target != nil {
                msg_send![NSApp(), targetForAction:action to:target from:&this]
            } else {
                msg_send![NSApp(), targetForAction:action]
            };
            let control: id = msg_send![this, view];
            let enabled: BOOL = msg_send![responder, validateUserInterfaceItem:this];
            msg_send![control, setEnabled:enabled];
        }
    }

    unsafe {
        class.add_method(sel!(validate), validate as extern fn(&mut Object, Sel));
    }

    class.register();
}
