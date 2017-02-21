use cocoa::appkit::*;
use cocoa::base::*;
use cocoa::foundation::*;
use std::os::raw::c_void;
use std::vec::Vec;
use std::ffi::CStr;
use libc;

mod app;
mod window;
mod toolbar;


use widgets::WidgetEvent;
// FIXME: can we pass directly WindowExt?
use window::GlutinWindow;
use window::WindowExt;

// FIXME: memory management is non existent.
// FIXME: use autorelease, retain and release (see Drop & IdRef)

pub struct Widgets {
    window: id,
    bottombar: id,
}

impl Widgets {

    pub fn setup_app() {
        app::setup();
    }

    pub fn new(window: &GlutinWindow) -> Widgets {
        unsafe {
            let winit_window = window.get_winit_window();
            let nswindow = winit_window.get_nswindow() as id;

            window::setup(nswindow);
            toolbar::setup(nswindow);

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
                window: nswindow,
                bottombar: bottombar,
            }
        }
    }

    fn get_toolbar_item(toolbar: id, identifier: &str) -> Option<id> {
        unsafe {
            let items: id = msg_send![toolbar, items];
            let count: NSInteger = msg_send![items, count];
            for i in 0..count {
                let item: id = msg_send![items, objectAtIndex:i];
                let item_identifier: id = msg_send![item, itemIdentifier];
                if NSString::isEqualToString(item_identifier, identifier) {
                    return Some(item);
                }
            }
            None
        }
    }

    #[allow(dead_code)]
    // FIXME: implement fmt::Debug instead
    fn print_nsview_tree(nsview: Option<id>, prefix: &str) {
        unsafe {
            let nsview = nsview.unwrap_or({
                let w: id = msg_send![NSApp(), keyWindow];
                let w: id = msg_send![w, contentView];
                let w: id = msg_send![w, superview];
                w
            });
            let classname = {
                let name: id = msg_send![nsview, className];
                let name: *const libc::c_char = msg_send![name, UTF8String];
                CStr::from_ptr(name).to_string_lossy().into_owned()
            };
            println!("{}{}", prefix, classname);

            let views: id = msg_send![nsview, subviews];
            let count: NSInteger = msg_send![views, count];

            for i in 0..count {
                let view: id = msg_send![views, objectAtIndex:i];
                Widgets::print_nsview_tree(Some(view), format!("-----{}", prefix).as_str());
            }
        }
    }

    pub fn get_events(&self) -> Vec<WidgetEvent> {
        unsafe {
            let delegate: id = msg_send![self.window, delegate];
            let event_queue: &mut Vec<WidgetEvent> = {
                let ivar: *mut c_void = *(&*delegate).get_ivar("event_queue");
                &mut *(ivar as *mut Vec<WidgetEvent>)
            };
            let clone = event_queue.clone();
            event_queue.clear();
            clone
        }
    }

    pub fn set_indicator_active(&self, active: bool) {
        unsafe {
            let toolbar = msg_send![self.window, toolbar];
            let item = Widgets::get_toolbar_item(toolbar, "indicator").unwrap();
            let indicator: id = msg_send![item, view];
            match active {
                true => msg_send![indicator, startAnimation:nil],
                false => msg_send![indicator, stopAnimation:nil],
            }
        }
    }

    pub fn set_back_button_enabled(&self, enabled: bool) {
        unsafe {
            let enabled = if enabled { YES } else { NO };
            let toolbar = msg_send![self.window, toolbar];
            let item = Widgets::get_toolbar_item(toolbar, "history").unwrap();
            let back_fwd_segment: id = msg_send![item, view];
            msg_send![back_fwd_segment, setEnabled:enabled forSegment:0];
        }
    }

    pub fn set_fwd_button_enabled(&self, enabled: bool) {
        unsafe {
            let enabled = if enabled { YES } else { NO };
            let toolbar = msg_send![self.window, toolbar];
            let item = Widgets::get_toolbar_item(toolbar, "history").unwrap();
            let back_fwd_segment: id = msg_send![item, view];
            msg_send![back_fwd_segment, setEnabled:enabled forSegment:1];
        }
    }

    pub fn set_urlbar_text(&self, text: &str) {
        // FIXME: also use nswindow.setRepresentedURL_
        unsafe {
            let toolbar = msg_send![self.window, toolbar];
            let item = Widgets::get_toolbar_item(toolbar, "urlbar").unwrap();
            let urlbar: id = msg_send![item, view];
            let string = NSString::alloc(nil).init_str(text);
            msg_send![urlbar, setStringValue:string];
        }
    }

    pub fn set_bottombar_text(&self, text: &str) {
        unsafe {
            let string = NSString::alloc(nil).init_str(text);
            NSTextField::setStringValue_(self.bottombar, string);
        }
    }
}
