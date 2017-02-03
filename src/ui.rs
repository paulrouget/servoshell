use cocoa::base::*;
use cocoa::foundation::*;
use cocoa::appkit::*;
use objc::runtime::{Class, Object, Sel, NO};
use objc::declare::ClassDecl;
use std::ops::Deref;

pub unsafe fn build_ui(window: id) {
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
                    NSToolbarFlexibleSpaceItemIdentifier,
                    NSString::alloc(nil).init_str("urlbar"),
                    NSToolbarFlexibleSpaceItemIdentifier,
                    NSToolbarToggleSidebarItemIdentifier,
                ])
            }
        }

        extern fn toolbar(this: &Object, _: Sel, _toolbar: id, identifier: id, _: id) -> id {
            unsafe {
                let mut item = nil;
                if NSString::isEqualToString(identifier, "reload") {
                    let button = NSView::init(NSButton::alloc(nil));
                    let label = NSString::alloc(nil).init_str("r");
                    NSButton::setBezelStyle_(button, NSBezelStyle::NSRoundedBezelStyle);
                    NSButton::setTitle_(button, label);
                    item = NSToolbarItem::alloc(nil).initWithItemIdentifier_(identifier).autorelease();
                    NSToolbarItem::setMinSize_(item, NSSize::new(35., 35.));
                    NSToolbarItem::setMaxSize_(item, NSSize::new(35., 35.));
                    NSToolbarItem::setView_(item, button);
                }
                if NSString::isEqualToString(identifier, "back") {
                    let button = NSView::init(NSButton::alloc(nil));
                    let label = NSString::alloc(nil).init_str("<");
                    NSButton::setBezelStyle_(button, NSBezelStyle::NSRoundedBezelStyle);
                    NSButton::setTitle_(button, label);
                    item = NSToolbarItem::alloc(nil).initWithItemIdentifier_(identifier).autorelease();
                    NSToolbarItem::setMinSize_(item, NSSize::new(35., 35.));
                    NSToolbarItem::setMaxSize_(item, NSSize::new(35., 35.));
                    NSToolbarItem::setView_(item, button);
                }
                if NSString::isEqualToString(identifier, "forward") {
                    let button = NSView::init(NSButton::alloc(nil));
                    let label = NSString::alloc(nil).init_str(">");
                    NSButton::setBezelStyle_(button, NSBezelStyle::NSRoundedBezelStyle);
                    NSButton::setTitle_(button, label);
                    item = NSToolbarItem::alloc(nil).initWithItemIdentifier_(identifier).autorelease();
                    NSToolbarItem::setMinSize_(item, NSSize::new(35., 35.));
                    NSToolbarItem::setMaxSize_(item, NSSize::new(35., 35.));
                    NSToolbarItem::setView_(item, button);
                }
                if NSString::isEqualToString(identifier, "urlbar") {
                    let field = NSView::init(NSTextField::alloc(nil));
                    let string = NSString::alloc(nil).init_str("foobar");
                    NSTextField::setStringValue_(field, string);
                    item = NSToolbarItem::alloc(nil).initWithItemIdentifier_(identifier).autorelease();
                    NSButton::setBezelStyle_(field, NSBezelStyle::NSRoundedBezelStyle);
                    NSToolbarItem::setMinSize_(item, NSSize::new(100., 0.));
                    NSToolbarItem::setMaxSize_(item, NSSize::new(400., 100.));
                    NSToolbarItem::setView_(item, field);
                }
                item
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
