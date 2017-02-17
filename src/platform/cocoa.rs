extern crate cocoa;
extern crate objc_foundation;


use self::cocoa::base::*;
use self::cocoa::foundation::*;
use self::cocoa::appkit::*;
use self::cocoa::base::id;
use self::objc_foundation::{INSObject, NSObject};
use objc::runtime::{Class, Object, Sel};
use objc::declare::ClassDecl;
use std::os::raw::c_void;
use std::vec::Vec;

use widgets::WidgetEvent;

// FIXME: can we pass directly WindowExt?
use window::GlutinWindow;
// Necessary for get_nswindow() traits
use window::WindowExt;

// FIXME: memory management is non existent.
// FIXME: use autorelease, retain and release (see Drop & IdRef)
// FIXME: move none native code to ../../widget.rs

/// WINDOW

struct ToolbarItems {
    reload_button: id,
    back_fwd_segment: id,
    urlbar: id,
    indicator: id,
}

pub struct Widgets {
    bottombar: id,
    event_queue_ptr: *mut Vec<WidgetEvent>,
    toolbar_items_ptr: *mut ToolbarItems,
}

impl Widgets {
    pub fn new(window: &GlutinWindow) -> Widgets {
        unsafe {
            let winit_window = window.get_winit_window();
            let nswindow = winit_window.get_nswindow() as id;

            Self::setup_window(nswindow);

            let event_queue = Box::new(Vec::new());

            // FIXME: initWithIdentifier_ <- can't we ust do a regular `init()`?
            let toolbar = NSToolbar::alloc(nil).autorelease();
            toolbar.initWithIdentifier_(NSString::alloc(nil).init_str("tb1"));

            let toolbar_items = Self::build_toolbar_items();
            let toolbar_items = Box::new(toolbar_items);

            // Handle toolbar construction
            let toolbar_delegate: id = msg_send![Class::get("ToolbarDelegate").unwrap(), new];
            // FIXME: When is setDelegate:nil called???
            let _: () = msg_send![toolbar, setDelegate: toolbar_delegate];

            // Handle clicks on items
            let target: id = msg_send![Class::get("UITarget").unwrap(), new];
            msg_send![toolbar_items.reload_button, setTarget: target];
            msg_send![toolbar_items.reload_button, setAction:sel![on_reload_click]];
            msg_send![toolbar_items.back_fwd_segment, setTarget: target];
            msg_send![toolbar_items.back_fwd_segment, setAction:sel![on_segment_click]];

            // FIXME: it's our job to destroy toolbar_items and event_queue
            // FIXME: I don't underdtand why we need to use box::into_raw here
            // and not a simple *mut.
            let toolbar_items_ptr = Box::into_raw(toolbar_items);
            let event_queue_ptr = Box::into_raw(event_queue);
            (*target).set_ivar("toolbar_items", toolbar_items_ptr as *mut c_void);
            (*target).set_ivar("event_queue", event_queue_ptr as *mut c_void);
            (*toolbar_delegate).set_ivar("toolbar_items", toolbar_items_ptr as *mut c_void);

            nswindow.setToolbar_(toolbar);

            let rect = NSRect::new(NSPoint::new(2., -2.), NSSize::new(400., 20.));
            let bottombar = NSTextField::alloc(nil).autorelease();
            NSView::initWithFrame_(bottombar, rect);
            msg_send![bottombar, setEditable: NO];
            msg_send![bottombar, setSelectable: NO];
            msg_send![bottombar, setBordered: NO];
            msg_send![bottombar, setBackgroundColor: NSColor::clearColor(nil)];

            let nsview = winit_window.get_nsview() as id;
            msg_send![nsview, addSubview: bottombar];

            Widgets {
                bottombar: bottombar,
                event_queue_ptr: event_queue_ptr,
                toolbar_items_ptr: toolbar_items_ptr,
            }
        }
    }

    pub fn get_events(&self) -> Vec<WidgetEvent> {
        unsafe {
            let ref mut event_queue = *self.event_queue_ptr;
            let clone = event_queue.clone();
            event_queue.clear();
            clone
        }
    }

    pub fn set_indicator_active(&self, active: bool) {
        unsafe {
            let ref toolbar_items = *self.toolbar_items_ptr;
            if active {
                toolbar_items.indicator.startAnimation_(nil);
            } else {
                toolbar_items.indicator.stopAnimation_(nil);
            }
        }
    }

    pub fn set_back_button_enabled(&self, enabled: bool) {
        let enabled = if enabled { YES } else { NO };
        unsafe {
            let ref toolbar_items = *self.toolbar_items_ptr;
            toolbar_items.back_fwd_segment.setEnabled_forSegment_(enabled, 0);
        }
    }

    pub fn set_fwd_button_enabled(&self, enabled: bool) {
        let enabled = if enabled { YES } else { NO };
        unsafe {
            let ref toolbar_items = *self.toolbar_items_ptr;
            toolbar_items.back_fwd_segment.setEnabled_forSegment_(enabled, 1);
        }
    }

    pub fn set_urlbar_text(&self, text: &str) {
        unsafe {
            let ref toolbar_items = *self.toolbar_items_ptr;
            let string = NSString::alloc(nil).init_str(text);
            NSTextField::setStringValue_(toolbar_items.urlbar, string);
        }
    }

    pub fn set_bottombar_text(&self, text: &str) {
        unsafe {
            let string = NSString::alloc(nil).init_str(text);
            NSTextField::setStringValue_(self.bottombar, string);
        }
    }

    pub fn setup_app() {
        declare_toolbar_delegate();
        declare_uitarget();
        unsafe {

            let quit_item = {
                let label = NSString::alloc(nil).init_str("Quit");
                let action = selector("terminate:");
                let key = NSString::alloc(nil).init_str("q");
                NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(label, action, key).autorelease()
            };

            let reload_item = {
                let label = NSString::alloc(nil).init_str("Reload");
                let action = selector("on_reload_click");
                let key = NSString::alloc(nil).init_str("r");
                NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(label, action, key).autorelease()
            };

            let go_back_item = {
                let label = NSString::alloc(nil).init_str("Back");
                let action = selector("go_back");
                let key = msg_send![class("NSString"), stringWithCharacters:&NSLeftArrowFunctionKey length:1];
                NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(label, action, key).autorelease()
            };

            let go_fwd_item = {
                let label = NSString::alloc(nil).init_str("Forward");
                let action = selector("go_fwd:");
                let key = msg_send![class("NSString"), stringWithCharacters:&NSRightArrowFunctionKey length:1];
                NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(label, action, key).autorelease()
            };

            let open_location_item = {
                let label = NSString::alloc(nil).init_str("Open Location");
                let action = selector("open_location");
                let key = NSString::alloc(nil).init_str("l");
                NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(label, action, key).autorelease()
            };


            let app_menu = NSMenu::new(nil).autorelease();
                app_menu.addItem_(quit_item);
            let app_menu_item = NSMenuItem::new(nil).autorelease();
                app_menu_item.setSubmenu_(app_menu);

            let file_menu = NSMenu::new(nil).initWithTitle_(NSString::alloc(nil).init_str("File")).autorelease();
                file_menu.addItem_(open_location_item);
                file_menu.setAutoenablesItems(NO);
            let file_menu_item = NSMenuItem::new(nil).autorelease();
                file_menu_item.setSubmenu_(file_menu);

            let view_menu = NSMenu::new(nil).initWithTitle_(NSString::alloc(nil).init_str("View")).autorelease();
                view_menu.addItem_(reload_item);
                view_menu.setAutoenablesItems(NO);
            let view_menu_item = NSMenuItem::new(nil).autorelease();
                view_menu_item.setSubmenu_(view_menu);

            let history_menu = NSMenu::new(nil).initWithTitle_(NSString::alloc(nil).init_str("History")).autorelease();
                history_menu.addItem_(go_back_item);
                history_menu.addItem_(go_fwd_item);
                history_menu.setAutoenablesItems(NO);
            let history_menu_item = NSMenuItem::new(nil).autorelease();
                history_menu_item.setSubmenu_(history_menu);

            let menubar = NSMenu::new(nil).autorelease();
                menubar.addItem_(app_menu_item);
                menubar.addItem_(file_menu_item);
                menubar.addItem_(view_menu_item);
                menubar.addItem_(history_menu_item);


            NSApp().setMainMenu_(menubar);

        }
    }

    fn setup_window(nswindow: id) {
        unsafe {
            nswindow.setTitleVisibility_(NSWindowTitleVisibility::NSWindowTitleHidden);
            let mask = nswindow.styleMask() as NSUInteger |
                       NSWindowMask::NSFullSizeContentViewWindowMask as NSUInteger;
            nswindow.setStyleMask_(mask);
            // FIXME: dark ui
            // nswindow.setAppearance_(NSAppearance::named_(nil, NSAppearanceNameVibrantDark));
        }
    }

    fn build_toolbar_items() -> ToolbarItems {
        unsafe {
            let reload_button = NSView::init(NSButton::alloc(nil));
            NSButton::setBezelStyle_(reload_button, NSBezelStyle::NSRoundedBezelStyle);
            NSButton::setImage_(reload_button,
                                NSImage::imageNamed_(nil, NSImageNameRefreshTemplate));

            let back_fwd_segment = NSView::init(NSSegmentedControl::alloc(nil));
            back_fwd_segment.setSegmentStyle_(NSSegmentStyle::NSSegmentStyleRounded);
            let mode = NSSegmentSwitchTrackingMode::NSSegmentSwitchTrackingMomentary;
            back_fwd_segment.setTrackingMode_(mode);
            back_fwd_segment.setSegmentCount_(2);
            let img_back = NSImage::imageNamed_(nil, NSImageNameGoLeftTemplate);
            let img_fwd = NSImage::imageNamed_(nil, NSImageNameGoRightTemplate);
            back_fwd_segment.setImage_forSegment_(img_back, 0);
            back_fwd_segment.setImage_forSegment_(img_fwd, 1);
            back_fwd_segment.setEnabled_forSegment_(NO, 0);
            back_fwd_segment.setEnabled_forSegment_(NO, 1);

            let urlbar = NSView::init(NSTextField::alloc(nil));
            msg_send![urlbar, setBezelStyle: NSBezelStyle::NSRoundedBezelStyle];

            // FIXME: magic value
            let rect = NSRect::new(NSPoint::new(0., 0.), NSSize::new(20., 20.));
            let indicator = NSProgressIndicator::initWithFrame_(NSProgressIndicator::alloc(nil),
                                                                rect);
            indicator.setStyle_(NSProgressIndicatorStyle::NSProgressIndicatorSpinningStyle);
            msg_send![indicator, setDisplayedWhenStopped: NO];


            ToolbarItems {
                reload_button: reload_button,
                back_fwd_segment: back_fwd_segment,
                urlbar: urlbar,
                indicator: indicator,
            }
        }
    }
}

extern "C" fn toolbar_allowed_item_identifiers(_this: &Object, _cmd: Sel, _toolbar: id) -> id {
    unsafe { NSArray::array(nil) }
}

extern "C" fn toolbar_default_item_identifiers(_this: &Object, _cmd: Sel, _toolbar: id) -> id {
    unsafe {
        // FIXME: could be static
        NSArray::arrayWithObjects(nil,
                                  &[NSString::alloc(nil).init_str("history"),
                                    NSString::alloc(nil).init_str("reload"),
                                    NSToolbarFlexibleSpaceItemIdentifier,
                                    NSString::alloc(nil).init_str("urlbar"),
                                    NSToolbarFlexibleSpaceItemIdentifier,
                                    NSString::alloc(nil).init_str("indicator"),
                                    NSToolbarToggleSidebarItemIdentifier])
    }
}

extern "C" fn build_toolbar_item(this: &Object,
                                 _cmd: Sel,
                                 _toolbar: id,
                                 identifier: id,
                                 _will_be_inserted: BOOL)
                                 -> id {
    let mut item = nil;

    unsafe {
        let toolbar_items: &ToolbarItems = {
            let ivar: *mut c_void = *this.get_ivar("toolbar_items");
            &*(ivar as *mut ToolbarItems)
        };
        // FIXME: magic values
        if NSString::isEqualToString(identifier, "indicator") {
            item = NSToolbarItem::alloc(nil).initWithItemIdentifier_(identifier).autorelease();
            NSToolbarItem::setMinSize_(item, NSSize::new(20., 20.));
            NSToolbarItem::setMaxSize_(item, NSSize::new(20., 20.));
            NSToolbarItem::setView_(item, toolbar_items.indicator);
        }
        if NSString::isEqualToString(identifier, "reload") {
            item = NSToolbarItem::alloc(nil).initWithItemIdentifier_(identifier).autorelease();
            NSToolbarItem::setMinSize_(item, NSSize::new(35., 35.));
            NSToolbarItem::setMaxSize_(item, NSSize::new(35., 35.));
            NSToolbarItem::setView_(item, toolbar_items.reload_button);
        }
        if NSString::isEqualToString(identifier, "history") {
            item = NSToolbarItem::alloc(nil).initWithItemIdentifier_(identifier).autorelease();
            NSToolbarItem::setMinSize_(item, NSSize::new(65., 25.));
            NSToolbarItem::setMaxSize_(item, NSSize::new(65., 40.));
            NSToolbarItem::setView_(item, toolbar_items.back_fwd_segment);
        }
        if NSString::isEqualToString(identifier, "urlbar") {
            item = NSToolbarItem::alloc(nil).initWithItemIdentifier_(identifier).autorelease();
            NSToolbarItem::setMinSize_(item, NSSize::new(100., 0.));
            NSToolbarItem::setMaxSize_(item, NSSize::new(400., 100.));
            NSToolbarItem::setView_(item, toolbar_items.urlbar);
        }
    }
    item
}

extern "C" fn on_reload_click(this: &Object, _cmd: Sel) {
    unsafe {
        let event_queue: &mut Vec<WidgetEvent> = {
            let ivar: *mut c_void = *this.get_ivar("event_queue");
            &mut *(ivar as *mut Vec<WidgetEvent>)
        };
        event_queue.push(WidgetEvent::ReloadClicked);
    }
}

extern "C" fn on_segment_click(this: &Object, _cmd: Sel) {
    unsafe {
        let toolbar_items: &ToolbarItems = {
            let ivar: *mut c_void = *this.get_ivar("toolbar_items");
            &*(ivar as *mut ToolbarItems)
        };
        let event_queue: &mut Vec<WidgetEvent> = {
            let ivar: *mut c_void = *this.get_ivar("event_queue");
            &mut *(ivar as *mut Vec<WidgetEvent>)
        };
        let idx: NSInteger = msg_send![toolbar_items.back_fwd_segment, selectedSegment];
        match idx {
            0 => event_queue.push(WidgetEvent::BackClicked),
            1 => event_queue.push(WidgetEvent::FwdClicked),
            _ => {}
        }
    }
}

fn declare_toolbar_delegate() {
    let superclass = NSObject::class();
    let mut decl = ClassDecl::new("ToolbarDelegate", superclass).unwrap();
    unsafe {
        decl.add_method(sel!(toolbarAllowedItemIdentifiers:),
                        toolbar_allowed_item_identifiers as extern "C" fn(&Object, Sel, id) -> id);
        decl.add_method(sel!(toolbarDefaultItemIdentifiers:),
                        toolbar_default_item_identifiers as extern "C" fn(&Object, Sel, id) -> id);
        decl.add_method(sel!(toolbar:itemForItemIdentifier:willBeInsertedIntoToolbar:),
                        build_toolbar_item as extern "C" fn(&Object, Sel, id, id, BOOL) -> id);
        decl.add_ivar::<*mut c_void>("toolbar_items");
    }
    decl.register();
}

fn declare_uitarget() {
    let superclass = NSObject::class();
    let mut decl = ClassDecl::new("UITarget", superclass).unwrap();
    unsafe {
        decl.add_method(sel!(on_reload_click),
                        on_reload_click as extern "C" fn(&Object, Sel));
        decl.add_method(sel!(on_segment_click),
                        on_segment_click as extern "C" fn(&Object, Sel));
        decl.add_ivar::<*mut c_void>("toolbar_items");
        decl.add_ivar::<*mut c_void>("event_queue");
    }
    decl.register();
}
