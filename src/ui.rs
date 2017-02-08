use std::sync::{Once, ONCE_INIT};
use cocoa::base::*;
use cocoa::foundation::*;
use cocoa::appkit::*;
use objc::runtime::{Class, Object, Sel};
use objc::declare::ClassDecl;
use std::ops::Deref;
use objc_foundation::{INSObject, NSObject};
use objc::Message;


const DARK: bool = false;

pub struct UI {
    reload_button: Option<id>,
    back_fwd_segment: Option<id>,
    urlbar: Option<id>,
}

impl UI {

    unsafe fn init_window(window: id) {
        window.setTitleVisibility_(NSWindowTitleVisibility::NSWindowTitleHidden);
        let mask = window.styleMask() as NSUInteger | NSWindowMask::NSFullSizeContentViewWindowMask as NSUInteger;
        window.setStyleMask_(mask);
        if DARK {
            window.setAppearance_(NSAppearance::named_(nil, NSAppearanceNameVibrantDark));
        }
    }

    pub fn new(window: id) -> UI {
        let mut ui = UI {
            reload_button: None,
            back_fwd_segment: None,
            urlbar: None,
        };

        unsafe {
            UI::init_window(window);

            let toolbar = NSToolbar::alloc(nil).initWithIdentifier_(NSString::alloc(nil).init_str("tb1"));
            toolbar.setDisplayMode_(NSToolbarDisplayMode::NSToolbarDisplayModeIconAndLabel);

            let reload_button = {
                let button = NSView::init(NSButton::alloc(nil));
                NSButton::setBezelStyle_(button, NSBezelStyle::NSRoundedBezelStyle);
                NSButton::setImage_(button, NSImage::imageNamed_(nil, NSImageNameRefreshTemplate));
                let handler = UIHandler::new();
                let _: () = msg_send![button, setTarget:handler];
                let _: () = msg_send![button, setAction:sel![on_reload_click]];
                button
            };

            let back_fwd_segment = {
                let db = NSView::init(NSSegmentedControl::alloc(nil));
                db.setSegmentStyle_(NSSegmentStyle::NSSegmentStyleRounded);
                db.setTrackingMode_(NSSegmentSwitchTrackingMode::NSSegmentSwitchTrackingMomentary);
                db.setSegmentCount_(2);
                db.setImage_forSegment_(NSImage::imageNamed_(nil, NSImageNameGoLeftTemplate), 0);
                db.setImage_forSegment_(NSImage::imageNamed_(nil, NSImageNameGoRightTemplate), 1);
                db
            };

            let urlbar = {
                let field = NSView::init(NSTextField::alloc(nil));
                // FIXME: lazy
                NSButton::setBezelStyle_(field, NSBezelStyle::NSRoundedBezelStyle);
                field
            };


            // FIXME: if s/td/_/ , no toolbar button show up
            let td = ToolbarDelegate::new(DelegateState {
                // FIXME: why clone?
                toolbar: IdRef::new(toolbar),
                reload_button: IdRef::new(reload_button),
                back_fwd_segment: IdRef::new(back_fwd_segment),
                urlbar: IdRef::new(urlbar),
            });

            window.setToolbar_(toolbar);

            ui = UI {
                reload_button: Some(reload_button),
                back_fwd_segment: Some(back_fwd_segment),
                urlbar: Some(urlbar),
            }
        }

        ui
    }

    pub fn set_textfield_text(&self, text: &str) {
        unsafe {
            let string = NSString::alloc(nil).init_str(text);
            NSTextField::setStringValue_(self.urlbar.unwrap(), string);
        }
    }
}


struct DelegateState {
    toolbar: IdRef,
    reload_button: IdRef,
    back_fwd_segment: IdRef,
    urlbar: IdRef,
}

struct ToolbarDelegate {
    state: Box<DelegateState>,
    _this: IdRef,
}

// FIXME: Why not http://sasheldon.com/rust-objc/objc_foundation/trait.INSObject.html
// Read more about objc_foundation.
impl ToolbarDelegate {
    fn class() -> *const Class {
        use std::os::raw::c_void;
        use std::sync::{Once, ONCE_INIT};

        extern fn toolbar_allowed_item_identifiers(_this: &Object, _: Sel, _: id) -> id {
            unsafe {
                NSArray::array(nil)
            }
        }

        extern fn toolbar_default_item_identifiers(_this: &Object, _: Sel, _: id) -> id {
            unsafe {
                NSArray::arrayWithObjects(nil, &[
                    NSString::alloc(nil).init_str("history"),
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
                let state: *mut c_void = *this.get_ivar("exampleState");
                let state = &mut *(state as *mut DelegateState);

                if NSString::isEqualToString(identifier, "reload") {
                    item = NSToolbarItem::alloc(nil).initWithItemIdentifier_(identifier).autorelease();
                    NSToolbarItem::setMinSize_(item, NSSize::new(35., 35.));
                    NSToolbarItem::setMaxSize_(item, NSSize::new(35., 35.));
                    NSToolbarItem::setView_(item, *state.reload_button);
                }
                if NSString::isEqualToString(identifier, "history") {
                    item = NSToolbarItem::alloc(nil).initWithItemIdentifier_(identifier).autorelease();
                    NSToolbarItem::setMinSize_(item, NSSize::new(65., 25.));
                    NSToolbarItem::setMaxSize_(item, NSSize::new(65., 40.));
                    NSToolbarItem::setView_(item, *state.back_fwd_segment);
                }

                if NSString::isEqualToString(identifier, "urlbar") {
                    item = NSToolbarItem::alloc(nil).initWithItemIdentifier_(identifier).autorelease();
                    NSToolbarItem::setMinSize_(item, NSSize::new(100., 0.));
                    NSToolbarItem::setMaxSize_(item, NSSize::new(400., 100.));
                    NSToolbarItem::setView_(item, *state.urlbar);
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



// See https://github.com/SSheldon/rust-objc-foundation/blob/master/examples/custom_class.rs
pub struct UIHandler {
}
impl UIHandler { }
unsafe impl Message for UIHandler { }
static UIHANDLER_REGISTER_CLASS: Once = ONCE_INIT;
impl INSObject for UIHandler {

    fn class() -> &'static Class {
        UIHANDLER_REGISTER_CLASS.call_once(|| {
            let superclass = NSObject::class();
            let mut decl = ClassDecl::new("UIHandler", superclass).unwrap();
            decl.add_ivar::<u32>("_number");
            extern fn on_reload_click(this: &Object, _cmd: Sel) {
                println!("RELOAD CLICK");
            }
            unsafe {
                let on_reload_click : extern fn(&Object, Sel) = on_reload_click;
                decl.add_method(sel!(on_reload_click), on_reload_click);
            }
            decl.register();
        });
        Class::get("UIHandler").unwrap()
    }
}
