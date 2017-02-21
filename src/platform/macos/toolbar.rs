use cocoa::appkit::*;
use cocoa::base::*;
use cocoa::foundation::*;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc_foundation::{INSObject, NSObject};
use std::os::raw::c_void;
use widgets::Widgets;

use std::sync::{Once, ONCE_INIT};
static START: Once = ONCE_INIT;

struct ToolbarItems {
    reload_button: id,
    back_fwd_segment: id,
    urlbar: id,
    indicator: id,
}

pub fn setup(nswindow: id) {
    unsafe {
        START.call_once(|| {
            register_toolbar_delegate();
            register_toolbar_target();
        });

        let toolbar_items = build_toolbar_items();
        // FIXME: using a target feels hacky. Is there a better way?
        let target: id = msg_send![Class::get("ToolbarTarget").unwrap(), new];
        msg_send![target, retain]; // FIXME: release?
        // FIXME: could use menuFormRepresentation
        msg_send![toolbar_items.reload_button, setTarget: target];
        msg_send![toolbar_items.reload_button, setAction:sel![reload_clicked]];
        msg_send![toolbar_items.back_fwd_segment, setTarget: target];
        msg_send![toolbar_items.back_fwd_segment, setAction:sel![segment_clicked]];

        let toolbar_items = Box::new(toolbar_items);

        // FIXME: it's our job to release toolbar_items
        let toolbar_items_ptr = Box::into_raw(toolbar_items);

        let delegate: id = msg_send![Class::get("ToolbarDelegate").unwrap(), new];
        (*delegate).set_ivar("toolbar_items", toolbar_items_ptr as *mut c_void);
        msg_send![delegate, retain]; // FIXME: release?
        // FIXME: When is setDelegate:nil called???

        let toolbar = NSToolbar::alloc(nil).autorelease();
        toolbar.initWithIdentifier_(NSString::alloc(nil).init_str("window_toolbar"));
        msg_send![toolbar, setDelegate:delegate];

        msg_send![nswindow, setToolbar:toolbar];
    }
}

fn build_toolbar_items() -> ToolbarItems {
    unsafe {
        let reload_button = {
            let view = NSView::init(NSButton::alloc(nil));
            NSButton::setBezelStyle_(view, NSBezelStyle::NSRoundedBezelStyle);
            NSButton::setImage_(view, NSImage::imageNamed_(nil, NSImageNameRefreshTemplate));
            view
        };

        let back_fwd_segment = {
            let view = NSView::init(NSSegmentedControl::alloc(nil));
            view.setSegmentStyle_(NSSegmentStyle::NSSegmentStyleRounded);
            let mode = NSSegmentSwitchTrackingMode::NSSegmentSwitchTrackingMomentary;
            view.setTrackingMode_(mode);
            view.setSegmentCount_(2);
            let img_back = NSImage::imageNamed_(nil, NSImageNameGoLeftTemplate);
            let img_fwd = NSImage::imageNamed_(nil, NSImageNameGoRightTemplate);
            view.setImage_forSegment_(img_back, 0);
            view.setImage_forSegment_(img_fwd, 1);
            view.setEnabled_forSegment_(NO, 0);
            view.setEnabled_forSegment_(NO, 1);
            view
        };

        let urlbar = {
            let view = NSView::init(NSTextField::alloc(nil));
            msg_send![view, setBezelStyle: NSBezelStyle::NSRoundedBezelStyle];
            view
        };

        let indicator = {
            // FIXME: magic value
            let rect = NSRect::new(NSPoint::new(0., 0.), NSSize::new(20., 20.));
            let view = NSProgressIndicator::initWithFrame_(NSProgressIndicator::alloc(nil), rect);
            view.setStyle_(NSProgressIndicatorStyle::NSProgressIndicatorSpinningStyle);
            msg_send![view, setDisplayedWhenStopped: NO];
            view
        };

        ToolbarItems {
            reload_button: reload_button,
            back_fwd_segment: back_fwd_segment,
            urlbar: urlbar,
            indicator: indicator,
        }
    }
}

fn register_toolbar_target() {
    unsafe {
        let superclass = NSObject::class();
        let mut decl = ClassDecl::new("ToolbarTarget", superclass).unwrap();
        decl.add_method(selector("reload_clicked"), reload_clicked as extern fn(&Object, Sel));
        decl.add_method(selector("segment_clicked"), segment_clicked as extern fn(&Object, Sel));
        decl.register();
    }
}

extern fn reload_clicked(_this: &Object, _cmd: Sel) {
    unsafe {
        let window: id = msg_send![NSApp(), keyWindow];
        let delegate: id = msg_send![window, delegate];
        msg_send![delegate, performSelector:selector("reload")];
    }
}

extern fn segment_clicked(_this: &Object, _cmd: Sel) {
    unsafe {
        let window: id = msg_send![NSApp(), keyWindow];
        let toolbar: id = msg_send![window, toolbar];
        let item = Widgets::get_toolbar_item(toolbar, "history").unwrap();
        let back_fwd_segment: id = msg_send![item, view];
        let delegate: id = msg_send![window, delegate];
        let idx: NSInteger = msg_send![back_fwd_segment, selectedSegment];
        match idx {
            0 => msg_send![delegate, performSelector:selector("go_back")],
            1 => msg_send![delegate, performSelector:selector("go_forward")],
            _ => {
                // FIXME: print warning
            }
        }
    }
}

fn register_toolbar_delegate() {
    unsafe {
        let superclass = NSObject::class();
        let mut decl = ClassDecl::new("ToolbarDelegate", superclass).unwrap();
        decl.add_method(selector("toolbarAllowedItemIdentifiers:"), toolbar_allowed_item_identifiers as extern fn(&Object, Sel, id) -> id);
        decl.add_method(selector("toolbarDefaultItemIdentifiers:"), toolbar_default_item_identifiers as extern fn(&Object, Sel, id) -> id);
        decl.add_method(selector("toolbar:itemForItemIdentifier:willBeInsertedIntoToolbar:"), build_toolbar_item as extern fn(&Object, Sel, id, id, BOOL) -> id);
        decl.add_ivar::<*mut c_void>("toolbar_items");
        decl.register();
    }
}


extern fn toolbar_allowed_item_identifiers(_this: &Object, _cmd: Sel, _toolbar: id) -> id {
    unsafe {
        NSArray::array(nil)
    }
}

extern fn toolbar_default_item_identifiers(_this: &Object, _cmd: Sel, _toolbar: id) -> id {
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

extern fn build_toolbar_item(this: &Object,
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
