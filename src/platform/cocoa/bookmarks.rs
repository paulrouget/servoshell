/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cocoa::base::*;
use cocoa::foundation::*;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};

pub fn register() {

    /* NShellBookmark */
    {
        let superclass = Class::get("NSObject").unwrap();
        let mut newclass = ClassDecl::new("NSShellBookmark", superclass).unwrap();
        newclass.add_ivar::<id>("link");
        newclass.add_ivar::<id>("name");
        newclass.register();
    }

    /* NSShellBookmarks */
    {
        let superclass = Class::get("NSObject").unwrap();
        let mut newclass = ClassDecl::new("NSShellBookmarks", superclass).unwrap();
        newclass.add_ivar::<id>("bookmarks");

        extern "C" fn awake_from_nib(_this: &mut Object, _sel: Sel) {}

        extern "C" fn child_of_item(_this: &Object,
                                    _sel: Sel,
                                    _outlineview: id,
                                    _index: NSInteger,
                                    _item: id)
                                    -> id {
            nil
        }

        extern "C" fn is_item_expandable(_this: &Object,
                                         _sel: Sel,
                                         _outlineview: id,
                                         _item: id)
                                         -> BOOL {
            NO
        }

        extern "C" fn number_of_child_of_item(_this: &Object,
                                              _sel: Sel,
                                              _outlineview: id,
                                              _item: id)
                                              -> NSInteger {
            0
        }

        extern "C" fn object_value(_this: &Object,
                                   _sel: Sel,
                                   _outlineview: id,
                                   _column: id,
                                   _item: id)
                                   -> id {
            nil
        }

        // FIXME: Yeah! Outlets, we want to use that everywhere instead of subviews
        // let textfield = msg_send![view, textField];

        unsafe {
            newclass.add_method(sel!(outlineView:child:ofItem:),
                                child_of_item as
                                extern "C" fn(&Object, Sel, id, NSInteger, id) -> id);
            newclass.add_method(sel!(outlineView:isItemExpandable:),
                                is_item_expandable as extern "C" fn(&Object, Sel, id, id) -> BOOL);
            newclass.add_method(sel!(outlineView:numberOfChildrenOfItem:),
                                number_of_child_of_item as
                                extern "C" fn(&Object, Sel, id, id) -> NSInteger);
            newclass.add_method(sel!(outlineView:objectValueForTableColumn:byItem:),
                                object_value as extern "C" fn(&Object, Sel, id, id, id) -> id);
            newclass.add_method(sel!(awakeFromNib),
                                awake_from_nib as extern "C" fn(&mut Object, Sel));
        }

        newclass.register();
    }


}
