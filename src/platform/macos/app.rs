use cocoa::appkit::*;
use cocoa::base::*;
use cocoa::foundation::*;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use std::os::raw::c_void;
use super::{View, Window};
use super::view;
use super::window;
use super::utils;
use std::sync::{Once, ONCE_INIT};

use app::AppEvent;

static INIT: Once = ONCE_INIT;

pub fn register() {
    unsafe {
        INIT.call_once(|| {

            let superclass = Class::get("NSObject").unwrap();
            let mut class = ClassDecl::new("NSShellApplicationDelegate", superclass).unwrap();
            class.add_ivar::<*mut c_void>("event_queue");

            class.add_method(sel!(applicationDidFinishLaunching:), did_finish_launching as extern fn(&Object, Sel, id));
            extern fn did_finish_launching(this: &Object, _sel: Sel, _notification: id) {
                utils::get_event_queue(this).push(AppEvent::DidFinishLaunching)
            }

            class.add_method(sel!(applicationDidChangeScreenParameter:), did_change_screen_parameter as extern fn(&Object, Sel, id));
            extern fn did_change_screen_parameter(this: &Object, _sel: Sel, _notification: id) {
                utils::get_event_queue(this).push(AppEvent::DidChangeScreenParameters)
            }

            class.add_method(sel!(applicationWillTerminate:), will_terminate as extern fn(&Object, Sel, id));
            extern fn will_terminate(this: &Object, _sel: Sel, _notification: id) {
                utils::get_event_queue(this).push(AppEvent::WillTerminate)
            }

            class.register();
        });
    }
}


pub struct App {
    nsapp: id
}

impl App {

    pub fn new() -> Result<App, &'static str> {

        // FIXME: move that earlier.
        register();
        view::register();
        window::register();

        let instances = match utils::load_nib("App.nib") {
            Ok(instances) => instances,
            Err(msg) => return Err(msg),
        };

        let nsapp = instances.into_iter().find(|i| {
            utils::id_is_instance_of(*i, "NSApplication")
        });

        let nsapp: id = match nsapp {
            None => return Err("Couldn't not find NSApplication instance in nib file"),
            Some(id) => id,
        };

        unsafe {
            nsapp.setActivationPolicy_(NSApplicationActivationPolicyRegular);
            let current_app = NSRunningApplication::currentApplication(nil);
            current_app.activateWithOptions_(NSApplicationActivateIgnoringOtherApps);
        }

        // FIXME: release and set delegate to nil
        let event_queue: Vec<AppEvent> = Vec::new();
        let event_queue_ptr = Box::into_raw(Box::new(event_queue));
        unsafe {
            let delegate: id = msg_send![class("NSShellApplicationDelegate"), alloc];
            (*delegate).set_ivar("event_queue", event_queue_ptr as *mut c_void);
            msg_send![nsapp, setDelegate:delegate];
        }

        Ok(App {
            nsapp: nsapp
        })
    }

    pub fn get_events(&self) -> Vec<AppEvent> {
        let nsobject = unsafe {
            let delegate: id = msg_send![self.nsapp, delegate];
            &*delegate
        };
        utils::get_event_queue(nsobject).drain(..).collect()
    }

    // Equivalent of NSApp.run()
    pub fn run<F>(&self, callback: F) where F: Fn() {

        unsafe { msg_send![self.nsapp, finishLaunching] };

        loop {
            unsafe {
                let pool = NSAutoreleasePool::new(nil);

                // Blocks until event available
                let nsevent = self.nsapp.nextEventMatchingMask_untilDate_inMode_dequeue_(
                    NSAnyEventMask.bits(),
                    NSDate::distantFuture(nil), NSDefaultRunLoopMode, YES);

                let event_type = nsevent.eventType() as u64;
                if event_type == NSApplicationDefined as u64 {
                    let event_subtype = nsevent.subtype() as i16;
                    if event_subtype == NSEventSubtype::NSApplicationActivatedEventType as i16 {
                        let nswindow: id = msg_send![nsevent, window];
                        msg_send![nswindow, eventLoopRised];
                    }
                } else {
                    msg_send![self.nsapp, sendEvent: nsevent];
                }

                // Get all pending events
                loop {
                    let nsevent = self.nsapp.nextEventMatchingMask_untilDate_inMode_dequeue_(
                        NSAnyEventMask.bits(),
                        NSDate::distantPast(nil), NSDefaultRunLoopMode, YES);
                    msg_send![self.nsapp, sendEvent: nsevent];
                    if nsevent == nil {
                        break;
                    }
                }

                msg_send![self.nsapp, updateWindows];
                msg_send![pool, release];
            }
            callback();
        }
    }

    pub fn create_window(&self) -> Result<(Window, View), &'static str> {
        let nswindow = match App::create_native_window() {
            Ok(w) => w,
            Err(msg) => return Err(msg),
        };
        let nsview = match App::find_nsservoview(nswindow) {
            Ok(v) => v,
            Err(msg) => return Err(msg),
        };
        Ok((Window::new(nswindow), View::new(nsview)))
    }

    fn create_native_window() -> Result<id, &'static str> {
        let instances = match utils::load_nib("Window.nib") {
            Ok(instances) => instances,
            Err(msg) => return Err(msg),
        };

        let mut nswindow: Option<id> = None;

        // FIXME: there's probably a more elegant way to do that
        for i in instances.into_iter() {
            unsafe {
                println!("Instance: {:?}", i);
                if utils::id_is_instance_of(i, "NSShellWindow") {
                    nswindow = Some(i);
                }
            }
        }

        let nswindow = match nswindow {
            None => return Err("Couldn't not find NSWindow instance in nib file"),
            Some(id) => id,
        };

        unsafe {
            nswindow.setTitleVisibility_(NSWindowTitleVisibility::NSWindowTitleHidden);
            let mask = nswindow.styleMask() as NSUInteger |
                       NSWindowMask::NSFullSizeContentViewWindowMask as NSUInteger;
            nswindow.setStyleMask_(mask);
        }

        Ok(nswindow)
    }



    fn find_nsservoview(nswindow: id) -> Result<id, &'static str> {
        // Depends on the Xib.
        // FIXME: search for identifier instead,
        // or maybe className
        Ok(unsafe {
            msg_send![nswindow, contentView]
        })
    }

}
