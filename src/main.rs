#![feature(box_syntax)]

#[macro_use]
extern crate objc;

extern crate cocoa;
extern crate objc_foundation;
extern crate libc;
extern crate rand;

mod nib;
mod initgl;
mod servoview;
mod servoengine;

use std::env::args;

use cocoa::appkit::*;
use cocoa::base::*;
use cocoa::foundation::*;

use servoengine::{ServoEngine, ServoEvent, FollowLinkPolicy};
use servoview::{ServoView, ViewEvent};

fn main() {

    let (nsapp, nswindow, nsview) = load_nib("ServoShellApp.nib");

    let servoview = ServoView::new(nsview);

    let servoengine = {
        let url = args().nth(1).unwrap_or("http://servo.org".to_owned());
        let geometry = servoview.get_geometry();
        let riser = servoview.create_eventloop_riser();
        // FIXME: hardcoded value
        let policy = FollowLinkPolicy::FollowOriginalDomain;
        ServoEngine::new(geometry, riser, &url, policy)
    };

    // Equivalent of NSApp.run()

    unsafe { msg_send![nsapp, finishLaunching] };
    loop {
        unsafe {
            let pool = NSAutoreleasePool::new(nil);

            // Blocks until event available
            let nsevent = nsapp.nextEventMatchingMask_untilDate_inMode_dequeue_(
                NSAnyEventMask.bits(),
                NSDate::distantFuture(nil), NSDefaultRunLoopMode, YES);

            let event_type = nsevent.eventType() as u64;
            if event_type == NSApplicationDefined as u64 {
                let event_subtype = nsevent.subtype() as i16;
                if event_subtype == NSEventSubtype::NSApplicationActivatedEventType as i16 {
                    let nswindow: id = msg_send![nsevent, window];
                    let view_tag: NSInteger = msg_send![nsevent, data1];
                    let content_view: id = msg_send![nswindow, contentView];
                    let nsview: id = msg_send![content_view, viewWithTag:view_tag];
                    msg_send![nsview, eventloopRised:nsevent];
                }
            } else {
                msg_send![nsapp, sendEvent:nsevent];
            }

            // Get all pending events
            loop {
                let nsevent = nsapp.nextEventMatchingMask_untilDate_inMode_dequeue_(
                    NSAnyEventMask.bits(),
                    NSDate::distantPast(nil), NSDefaultRunLoopMode, YES);
                msg_send![nsapp, sendEvent:nsevent];
                if nsevent == nil {
                    break
                }
            }

            msg_send![nsapp, updateWindows];
            msg_send![pool, release];
        }

        for e in servoview.get_events().into_iter() {
            match e {
                ViewEvent::Rised => {
                    servoview.swap_buffers();
                    servoengine.sync();
                },
                _ => {
                    println!("{:?}", e);
                }
            }
        }

        for e in servoengine.get_events().into_iter() {
            match e {
                ServoEvent::Present => {
                    servoview.swap_buffers();
                }
                _ => {
                    println!("{:?}", e);
                }
            }
        }

    }
}

fn load_nib(path: &str) -> (id, id, id) /* (nsapp, nswindow, nsview) */ {
    let (nsapp, nswindow) = {

        servoview::register_nsservoview();

        let instances = nib::load(path).unwrap();

        let mut nsapp: Option<id> = None;
        let mut nswindow: Option<id> = None;

        fn is_instance_of(i: id, classname: &'static str) -> bool {
            let is_instance: BOOL = unsafe {
                let classname = class(classname);
                msg_send![i, isKindOfClass:classname]
            };
            is_instance == YES
        };

        // FIXME: there's probably a more elegant way to do that
        for i in instances.into_iter() {
            unsafe {
                use std::ffi::CStr;
                let classname: id = msg_send![i, className];
                let classname: *const libc::c_char = msg_send![classname, UTF8String];
                let classname = CStr::from_ptr(classname).to_string_lossy().into_owned();
                println!("Found object {:?}", classname);

                if is_instance_of(i, "NSWindow") {
                    nswindow = Some(i);
                }
                if is_instance_of(i, "NSApplication") {
                    nsapp = Some(i);
                }
            }
        }

        let nsapp: id = match nsapp {
            None => panic!("Couldn't not find NSApplication instance in Nib file"),
            Some(id) => id
        };

        let nswindow: id = match nswindow {
            None => panic!("Couldn't not find NSWindow instance in Nib file"),
            Some(id) => id
        };

        (nsapp, nswindow)
    };

    unsafe {
        nsapp.setActivationPolicy_(NSApplicationActivationPolicyRegular);
        let current_app = NSRunningApplication::currentApplication(nil);
        current_app.activateWithOptions_(NSApplicationActivateIgnoringOtherApps);

        nswindow.setTitleVisibility_(NSWindowTitleVisibility::NSWindowTitleHidden);
        let mask = nswindow.styleMask() as NSUInteger | NSWindowMask::NSFullSizeContentViewWindowMask as NSUInteger;
        nswindow.setStyleMask_(mask);
    }

    let nsview: id = unsafe { msg_send![nswindow, contentView] };

    (nsapp, nswindow, nsview)
}
