use cocoa::base::*;
use cocoa::foundation::*;
use cocoa::appkit::*;
use objc::runtime::{Class, Object, Sel, NO};
use objc::declare::ClassDecl;
use std::ops::Deref;

pub unsafe fn build_ui(window: id) {


    // unsafe {
    //     let (width, height) = glutin_window.get_inner_size().expect("Failed to get window inner size.");
    //     let padding = 9.;

    //     let button_height = TOOLBAR_HEIGHT - 2. * padding;
    //     let button_width = 3. * button_height;

    //     {
    //         // Urlbar
    //         let origin = NSPoint::new(2. * padding + button_width + 50., height as f64 - TOOLBAR_HEIGHT + padding);
    //         let size = NSSize::new(width as f64 - 3. * padding - button_width - 50., TOOLBAR_HEIGHT - 2. * padding);
    //         let frame = NSRect::new(origin, size);
    //         let field = NSTextField::alloc(nil);
    //         NSTextField::initWithFrame_(field, frame);
    //         NSTextField::setStringValue_(field, NSString::alloc(nil).init_str("https://servo.org"));
    //         field.setEditable_(YES);
    //         nsview.addSubview_(field);
    //     }

    //     {
    //         // Reload button
    //         let origin = NSPoint::new(padding + 50., height as f64 - TOOLBAR_HEIGHT + padding);
    //         let size = NSSize::new(button_width, button_height);
    //         let frame = NSRect::new(origin, size);
    //         let button = NSButton::alloc(nil);
    //         NSButton::initWithFrame_(button, frame);
    //         NSButton::setBezelStyle_(button, NSBezelStyle::NSRoundedBezelStyle);
    //         NSButton::setTitle_(button, NSString::alloc(nil).init_str("reload"));
    //         nsview.addSubview_(button);
    //     }
    // }



    window.setTitleVisibility_(NSWindowTitleVisibility::NSWindowTitleHidden);

    let toolbar = NSToolbar::alloc(nil).initWithIdentifier_(NSString::alloc(nil).init_str("tb1"));
    toolbar.setDisplayMode_(NSToolbarDisplayMode::NSToolbarDisplayModeIconAndLabel);
    let toolbar_p = IdRef::new(toolbar);

    let td = ToolbarDelegate::new(DelegateState {
        toolbar: toolbar_p.clone(),
    });

    window.setToolbar_(toolbar);


}

struct DelegateState {
    toolbar: IdRef,
}

struct ToolbarDelegate {
    state: Box<DelegateState>,
    _this: IdRef,
}

impl ToolbarDelegate {
    fn class() -> *const Class {
        use std::os::raw::c_void;
        use std::sync::{Once, ONCE_INIT};

        extern fn toolbar_allowed_item_identifiers(this: &Object, _: Sel, _: id) -> id {
            unsafe {
                NSArray::array(nil)
            }
        }

        extern fn toolbar_default_item_identifiers(this: &Object, _: Sel, _: id) -> id {
            unsafe {
                NSArray::arrayWithObjects(nil, &[
                    NSString::alloc(nil).init_str("back"),
                    NSString::alloc(nil).init_str("forward"),
                    NSString::alloc(nil).init_str("reload"),
                    NSString::alloc(nil).init_str("urlbar"),
                ])
            }
        }

        extern fn toolbar(this: &Object, _: Sel, _: id, identifier: id, _: id) -> id {
            unsafe {
                if NSString::isEqualToString(identifier, "reload") {
                    let origin = NSPoint::new(0., 0.);
                    let size = NSSize::new(80., 40.);
                    let frame = NSRect::new(origin, size);
                    let button = NSButton::alloc(nil);
                    NSButton::initWithFrame_(button, frame);
                    NSButton::setBezelStyle_(button, NSBezelStyle::NSRoundedBezelStyle);
                    NSButton::setTitle_(button, NSString::alloc(nil).init_str("reload"));
                    let item = NSToolbarItem::alloc(nil).initWithItemIdentifier_(identifier).autorelease();
                    NSToolbarItem::setView_(item, button);
                    item
                } else {
                    if NSString::isEqualToString(identifier, "urlbar") {
                        let origin = NSPoint::new(45., 0.);
                        let size = NSSize::new(200., 30.);
                        let frame = NSRect::new(origin, size);
                        let field = NSTextField::alloc(nil);
                        NSTextField::initWithFrame_(field, frame);
                        NSTextField::setStringValue_(field, NSString::alloc(nil).init_str("https://servo.org"));
                        field.setEditable_(YES);
                        let item = NSToolbarItem::alloc(nil).initWithItemIdentifier_(identifier).autorelease();
                        NSToolbarItem::setView_(item, field);
                        item
                    } else {
                        nil
                    }
                }
            }
        }

        static mut DELEGATE_CLASS: *const Class = 0 as *const Class;
        static INIT: Once = ONCE_INIT;
        INIT.call_once(|| unsafe {
            let superclass = Class::get("NSObject").unwrap();
            let mut decl = ClassDecl::new("ExampleToolbarDelegate", superclass).unwrap();
            decl.add_method(sel!(toolbarAllowedItemIdentifiers:), toolbar_allowed_item_identifiers as extern fn(&Object, Sel, id) -> id);
            decl.add_method(sel!(toolbarDefaultItemIdentifiers:), toolbar_default_item_identifiers as extern fn(&Object, Sel, id) -> id);
            decl.add_method(sel!(toolbar:itemForItemIdentifier:willBeInsertedIntoToolbar:), toolbar as extern fn(&Object, Sel, id, id, id) -> id);
            decl.add_ivar::<*mut c_void>("exampleState");
            DELEGATE_CLASS = decl.register();
        });
        unsafe {
            DELEGATE_CLASS
        }
    }

    fn new(state: DelegateState) -> ToolbarDelegate {
        // Box the state so we can give a pointer to it
        let mut state = Box::new(state);
        let state_ptr: *mut DelegateState = &mut *state;
        unsafe {
            let delegate = IdRef::new(msg_send![ToolbarDelegate::class(), new]);
            (&mut **delegate).set_ivar("exampleState", state_ptr as *mut ::std::os::raw::c_void);
            let _: () = msg_send![*state.toolbar, setDelegate:*delegate];
            ToolbarDelegate { state: state, _this: delegate }
        }
    }
}

impl Drop for ToolbarDelegate {
    fn drop(&mut self) {
        unsafe {
            // Nil the toolbar's delegate so it doesn't still reference us
            let _: () = msg_send![*self.state.toolbar, setDelegate:nil];
        }
    }
}


struct IdRef(id);

impl IdRef {
    fn new(i: id) -> IdRef {
        IdRef(i)
    }

    #[allow(dead_code)]
    fn retain(i: id) -> IdRef {
        if i != nil {
            let _: id = unsafe { msg_send![i, retain] };
        }
        IdRef(i)
    }
}

impl Drop for IdRef {
    fn drop(&mut self) {
        if self.0 != nil {
            let _: () = unsafe { msg_send![self.0, release] };
        }
    }
}

impl Deref for IdRef {
    type Target = id;
    fn deref<'a>(&'a self) -> &'a id {
        &self.0
    }
}

impl Clone for IdRef {
    fn clone(&self) -> IdRef {
        if self.0 != nil {
            let _: id = unsafe { msg_send![self.0, retain] };
        }
        IdRef(self.0)
    }
}
