use libc;
use cocoa::appkit::*;
use cocoa::base::*;
use cocoa::foundation::*;
use std::ffi::CStr;
use std::process;

use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc_foundation::{INSObject, NSObject};

pub fn load() -> (id, id) {
    unsafe {
        // xib to nib: ibtool foobar.xib --compile foobar.nib
        let filename = NSString::alloc(nil).init_str("ServoShellApp.nib");
        let nsdata: id = msg_send![class("NSData"), dataWithContentsOfFile: filename];
        let nsnib: id = msg_send![class("NSNib"), alloc];
        msg_send![nsnib, initWithNibData:nsdata bundle:nil];

        let instances: id = msg_send![class("NSArray"), alloc];
        msg_send![instances, init];

        let success: BOOL = msg_send![nsnib, instantiateWithOwner:nil topLevelObjects:&instances];
        if success == NO {
            // Failed to load
            process::exit(1);
        }

        let count: NSInteger = msg_send![instances, count];

        let mut app: Option<id> = None;
        let mut win: Option<id> = None;

        for i in 0..count {
            let instance: id = msg_send![instances, objectAtIndex: i];
            let classname: id = msg_send![instance, className];
            let classname: *const libc::c_char = msg_send![classname, UTF8String];
            let classname = CStr::from_ptr(classname).to_string_lossy().into_owned();
            println!("instances[{}] is {}", i, classname);

            let is_app: BOOL = msg_send![instance, isKindOfClass:class("NSApplication")];
            if is_app == YES {
                app = Some(instance);
            }

            let is_win: BOOL = msg_send![instance, isKindOfClass:class("NSWindow")];
            if is_win == YES {
                win = Some(instance);
            }
        }

        (app.unwrap(), win.unwrap())
    }
}
