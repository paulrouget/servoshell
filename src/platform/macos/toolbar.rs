/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cocoa::foundation::*;
use cocoa::base::*;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use super::get_state;

pub fn register() {

    /* NSShellSegmentedCell */ {
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

    /* NSShellTextFieldCell */ {
        let superclass = Class::get("NSTextFieldCell").unwrap();
        let mut newclass = ClassDecl::new("NSShellTextFieldCell", superclass).unwrap();

        extern fn draw(this: &Object, _sel: Sel, rect: NSRect, view: id) {
            unsafe {
                let dark = get_state().dark_theme;
                let superclass = Class::get("NSTextFieldCell").unwrap();
                if dark {
                    msg_send![super(this, superclass), drawInteriorWithFrame:rect inView:view];
                } else {
                    msg_send![super(this, superclass), drawWithFrame:rect inView:view];
                }
            }
        }

        unsafe {
            newclass.add_method(sel!(drawWithFrame:inView:), draw as extern fn(&Object, Sel, NSRect, id));
        }

        newclass.register();
    }
}
