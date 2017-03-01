use libc;
use cocoa::appkit::*;
use cocoa::base::*;
use cocoa::foundation::*;
use platform::ServoView;
use super::servo_view::register_nsservoview;
use super::utils;

pub fn load() -> Result<(), &'static str> {

    register_nsservoview();

    let instances = match utils::load_nib("App.nib") {
        Ok(instances) => instances,
        Err(msg) => return Err(msg),
    };

    let mut nsapp: Option<id> = None;

    // fixme: there's probably a more elegant way to do that
    for i in instances.into_iter() {
        unsafe {
            use std::ffi::CStr;
            let classname: id = msg_send![i, className];
            let classname: *const libc::c_char = msg_send![classname, UTF8String];
            let classname = CStr::from_ptr(classname).to_string_lossy().into_owned();
            println!("found object {:?}", classname);
            if utils::id_is_instance_of(i, "NSApplication") {
                nsapp = Some(i);
            }
        }
    }

    let nsapp: id = match nsapp {
        None => return Err("Couldn't not find NSApplication instance in nib file"),
        Some(id) => id
    };

    unsafe {
        nsapp.setActivationPolicy_(NSApplicationActivationPolicyRegular);
        let current_app = NSRunningApplication::currentApplication(nil);
        current_app.activateWithOptions_(NSApplicationActivateIgnoringOtherApps);
    }

    Ok(())
}

// Equivalent of NSApp.run()
pub fn run<F>(callback: F) where F: Fn() {

    let nsapp = unsafe { NSApp() };

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

        callback();

    }
}

pub fn new_window() -> Result<ServoView, &'static str> {
    let nswindow = match create_native_window() {
        Ok(w) => w,
        Err(msg) => return Err(msg),
    };
    let nsview = match find_nsservoview(nswindow) {
        Ok(v) => v,
        Err(msg) => return Err(msg),
    };
    Ok(ServoView::new(nsview))
}

fn find_nsservoview(nswindow: id) -> Result<id, &'static str> {
    // Depends on the Xib.
    // FIXME: search for identifier instead,
    // or maybe className
    Ok(unsafe {nswindow.contentView()})
}

fn create_native_window() -> Result<id, &'static str> {

    let instances = match utils::load_nib("Window.nib") {
        Ok(instances) => instances,
        Err(msg) => return Err(msg),
    };

    let mut nswindow: Option<id> = None;

    // fixme: there's probably a more elegant way to do that
    for i in instances.into_iter() {
        unsafe {
            use std::ffi::CStr;
            let classname: id = msg_send![i, className];
            let classname: *const libc::c_char = msg_send![classname, UTF8String];
            let classname = CStr::from_ptr(classname).to_string_lossy().into_owned();
            println!("found object {:?}", classname);

            if utils::id_is_instance_of(i, "NSWindow") {
                nswindow = Some(i);
            }
        }
    }

    let nswindow = match nswindow {
        None => return Err("Couldn't not find NSWindow instance in nib file"),
        Some(id) => id
    };

    unsafe {
        nswindow.setTitleVisibility_(NSWindowTitleVisibility::NSWindowTitleHidden);
        let mask = nswindow.styleMask() as NSUInteger | NSWindowMask::NSFullSizeContentViewWindowMask as NSUInteger;
        nswindow.setStyleMask_(mask);
    }

    Ok(nswindow)
}
