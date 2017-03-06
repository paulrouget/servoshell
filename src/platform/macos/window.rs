use cocoa::appkit::*;
use cocoa::foundation::*;
use cocoa::base::*;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use std::os::raw::c_void;
use std::sync::{Once, ONCE_INIT};
use super::utils;

use window::WindowEvent;

static INIT: Once = ONCE_INIT;

pub fn register() {
    unsafe {
        INIT.call_once(|| {

            /* NSWindow subclass */ {

                let superclass = Class::get("NSWindow").unwrap();
                let mut class = ClassDecl::new("NSShellWindow", superclass).unwrap();
                class.add_ivar::<*mut c_void>("event_queue");

                class.add_method(sel!(toggleTabBar:), toggle_tabbar as extern fn(&Object, Sel, id));
                extern fn toggle_tabbar(this: &Object, _sel: Sel, sender: id) {
                    unsafe {
                        msg_send![super(this, Class::get("NSWindow").unwrap()), toggleTabBar:sender];
                    }
                    utils::get_event_queue(this).push(WindowEvent::GeometryDidChange);
                }

                class.add_method(sel!(toggleToolbarShown:), toggle_toolbar as extern fn(&Object, Sel, id));
                extern fn toggle_toolbar(this: &Object, _sel: Sel, sender: id) {
                    unsafe {
                        msg_send![super(this, Class::get("NSWindow").unwrap()), toggleToolbarShown:sender];
                    }
                    utils::get_event_queue(this).push(WindowEvent::GeometryDidChange);
                }

                class.add_method(sel!(eventLoopRised), event_loop_rised as extern fn(&Object, Sel));
                extern fn event_loop_rised(this: &Object, _sel: Sel) {
                    utils::get_event_queue(this).push(WindowEvent::EventLoopRised);
                }

                class.add_method(sel!(awakeFromNib), awake_from_nib as extern fn(&mut Object, Sel));
                extern fn awake_from_nib(this: &mut Object, _sel: Sel) {
                    let event_queue: Vec<WindowEvent> = Vec::new();
                    // FIXME: is that the best way to create a raw pointer?
                    let event_queue_ptr = Box::into_raw(Box::new(event_queue));
                    unsafe {
                        this.set_ivar("event_queue", event_queue_ptr as *mut c_void);
                    }
                }
                class.register();
            }

            /* NSWindowDelegate */ {

                let superclass = Class::get("NSObject").unwrap();
                let mut class = ClassDecl::new("NSShellWindowDelegate", superclass).unwrap();
                class.add_ivar::<*mut c_void>("event_queue");

                class.add_method(sel!(windowDidResize:), did_resize as extern fn(&Object, Sel, id));
                extern fn did_resize(this: &Object, _sel: Sel, _notification: id) {
                    utils::get_event_queue(this).push(WindowEvent::GeometryDidChange)
                }

                class.add_method(sel!(reloadClicked), reload_clicked as extern fn(&Object, Sel));
                extern fn reload_clicked(this: &Object, _sel: Sel) {
                    utils::get_event_queue(this).push(WindowEvent::ReloadClicked)
                }

                class.register();
            }
        });
    }
}


pub struct Window {
    nswindow: id,
}

impl Window {
    pub fn new(nswindow: id) -> Window {

        unsafe {
            // FIXME: release and set delegate to nil
            let event_queue_ptr: *mut c_void = *(&*nswindow).get_ivar("event_queue");
            let delegate: id = msg_send![class("NSShellWindowDelegate"), alloc];
            (*delegate).set_ivar("event_queue", event_queue_ptr);
            msg_send![nswindow, setDelegate:delegate];

            let toolbar: id = msg_send![nswindow, toolbar];
            let items: id = msg_send![toolbar, items];
            let count: NSInteger = msg_send![items, count];
            for i in 0..count {
                let item: id = msg_send![items, objectAtIndex:i];
                // FIXME: why delegate and not nswindow?
                Window::bind_toolbaritem(item, delegate);
            }
        }

        Window {
            nswindow: nswindow,
        }
    }

    fn bind_toolbaritem(item: id, delegate: id) {
        unsafe {
            let item_identifier: id = msg_send![item, itemIdentifier];

            if NSString::isEqualToString(item_identifier, "ShellToolbarReloadItem") {
                msg_send![item, setAction:sel!(reloadClicked)];
                msg_send![item, setTarget:delegate];
            }

            if NSString::isEqualToString(item_identifier, "ShellToolbarStopItem") {
                msg_send![item, setAction:sel!(reloadClicked)];
                msg_send![item, setTarget:delegate];
            }

            if NSString::isEqualToString(item_identifier, "ShellToolbarHistoryItem") {
                msg_send![item, setAction:sel!(historyClicked)];
                msg_send![item, setTarget:delegate];
            }

            if NSString::isEqualToString(item_identifier, "ShellToolbarZoomItem") {
                msg_send![item, setAction:sel!(zoomClicked)];
                msg_send![item, setTarget:delegate];
            }

        }
    }

    pub fn get_events(&self) -> Vec<WindowEvent> {
        let nsobject = unsafe { &*self.nswindow};
        utils::get_event_queue(nsobject).drain(..).collect()
    }

    pub fn set_url(&self, url: &str) {
        // FIXME: can't get NSWindow::representedURL to work
    }

    pub fn set_title(&self, title: &str) {
        unsafe {
            let title = NSString::alloc(nil).init_str(title);
            msg_send![self.nswindow, setTitle:title]
        }
    }

    pub fn create_eventloop_riser(&self) -> EventLoopRiser {
        let window_number: NSInteger = unsafe {
            msg_send![self.nswindow, windowNumber]
        };
        EventLoopRiser {
            window_number: window_number,
        }
    }
}

pub struct EventLoopRiser {
    window_number: NSInteger,
}

impl EventLoopRiser {
    pub fn rise(&self) {
        unsafe {
            let pool = NSAutoreleasePool::new(nil);
            let event: id = msg_send![class("NSEvent"),
                    otherEventWithType:NSApplicationDefined
                    location:NSPoint::new(0.0, 0.0)
                    modifierFlags:NSEventModifierFlags::empty()
                    timestamp:0.0
                    windowNumber:self.window_number
                    context:nil
                    subtype:NSEventSubtype::NSApplicationActivatedEventType
                    data1:0
                    data2:0];
            msg_send![event, retain];
            msg_send![NSApp(), postEvent:event atStart:NO];
            NSAutoreleasePool::drain(pool);
        }
    }
    pub fn clone(&self) -> EventLoopRiser {
        EventLoopRiser {
            window_number: self.window_number,
        }
    }
}
