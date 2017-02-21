use cocoa::appkit::*;
use cocoa::base::*;
use cocoa::foundation::*;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc_foundation::{INSObject, NSObject};
use std::os::raw::c_void;
use std::vec::Vec;

use widgets::WidgetEvent;

use std::sync::{Once, ONCE_INIT};
static START: Once = ONCE_INIT;

pub fn setup(nswindow: id) {
    unsafe {
        START.call_once(|| { register_window_delegate(); });

        nswindow.setTitleVisibility_(NSWindowTitleVisibility::NSWindowTitleHidden);
        let mask = nswindow.styleMask() as NSUInteger |
                   NSWindowMask::NSFullSizeContentViewWindowMask as NSUInteger;
        nswindow.setStyleMask_(mask);

        // FIXME: dark ui
        // nswindow.setAppearance_(NSAppearance::named_(nil, NSAppearanceNameVibrantDark));

        let event_queue: Box<Vec<WidgetEvent>> = Box::new(Vec::new());
        let event_queue_ptr = Box::into_raw(event_queue);

        let delegate: id = msg_send![Class::get("WindowDelegate").unwrap(), new];
        (*delegate).set_ivar("event_queue", event_queue_ptr as *mut c_void);
        msg_send![delegate, retain]; // FIXME: release?
        // FIXME: When is setDelegate:nil called???
        msg_send![nswindow, setDelegate:delegate];
    }
}

fn register_window_delegate() {
    unsafe {
        let superclass = NSObject::class();
        let mut decl = ClassDecl::new("WindowDelegate", superclass).unwrap();
        decl.add_method(selector("reload"), reload as extern "C" fn(&Object, Sel));
        decl.add_method(selector("go_back"), go_back as extern "C" fn(&Object, Sel));
        decl.add_method(selector("go_forward"),
                        go_forward as extern "C" fn(&Object, Sel));
        decl.add_method(selector("open_location"),
                        open_location as extern "C" fn(&Object, Sel));
        decl.add_ivar::<*mut c_void>("event_queue");
        decl.register();
    }
}

extern "C" fn reload(this: &Object, _cmd: Sel) {
    println!("window delegate: on_reload");
    unsafe {
        let event_queue: &mut Vec<WidgetEvent> = {
            let ivar: *mut c_void = *this.get_ivar("event_queue");
            &mut *(ivar as *mut Vec<WidgetEvent>)
        };
        event_queue.push(WidgetEvent::ReloadClicked);
    }
}

extern "C" fn go_back(this: &Object, _cmd: Sel) {
    println!("window delegate: on_go_back");
    unsafe {
        let event_queue: &mut Vec<WidgetEvent> = {
            let ivar: *mut c_void = *this.get_ivar("event_queue");
            &mut *(ivar as *mut Vec<WidgetEvent>)
        };
        event_queue.push(WidgetEvent::BackClicked);
    }
}

extern "C" fn go_forward(this: &Object, _cmd: Sel) {
    println!("window delegate: on_go_forward");
    unsafe {
        let event_queue: &mut Vec<WidgetEvent> = {
            let ivar: *mut c_void = *this.get_ivar("event_queue");
            &mut *(ivar as *mut Vec<WidgetEvent>)
        };
        event_queue.push(WidgetEvent::FwdClicked);
    }
}

extern "C" fn open_location(_this: &Object, _cmd: Sel) {
    println!("window delegate: on_open_location");
}
