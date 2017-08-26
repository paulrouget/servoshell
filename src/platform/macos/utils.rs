/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use state::AppState;
use cocoa::base::*;
use cocoa::appkit::*;
use cocoa::foundation::*;
use objc::runtime::Object;
use std::os::raw::c_void;
use std::ffi::CStr;
use app::App;
use libc;

pub fn load_nib(filename: &str) -> Result<Vec<id>, &'static str> {

    let path = App::get_nibs_path().unwrap();
    let path = path.join(filename);
    let path = path.to_str().unwrap();

    unsafe {
        let filename = NSString::alloc(nil).init_str(path);
        let nsdata: id = msg_send![class("NSData"), dataWithContentsOfFile: filename];
        let nsnib: id = msg_send![class("NSNib"), alloc];
        msg_send![nsnib, initWithNibData:nsdata bundle:nil];

        let objects: id = msg_send![class("NSArray"), alloc];
        msg_send![objects, init];

        let success: BOOL = msg_send![nsnib, instantiateWithOwner:nil topLevelObjects:&objects];
        if success == NO {
            return Err("Can't load nib file");
        }

        let count: NSInteger = msg_send![objects, count];

        let mut instances = Vec::new();

        for i in 0..count {
            let instance: id = msg_send![objects, objectAtIndex:i];
            instances.push(instance);
        }

        Ok(instances)
    }
}

pub fn id_is_instance_of(id: id, classname: &'static str) -> bool {
    let is_instance: BOOL = unsafe {
        let classname = class(classname);
        msg_send![id, isKindOfClass:classname]
    };
    is_instance == YES
}

pub fn get_event_queue<T>(obj: &Object) -> &mut Vec<T> {
    get_ivar(obj, "event_queue")
}

pub fn get_ivar<'a, T>(obj: &'a Object, var: &'static str) -> &'a mut T {
    unsafe {
        let ivar: *mut c_void = *obj.get_ivar(var);
        &mut *(ivar as *mut T)
    }
}

// FIXME: Is there a better way?
#[allow(dead_code)]
pub fn get_classname(id: id) -> String {
    unsafe {
        let name: id = msg_send![id, className];
        let name: *const libc::c_char = msg_send![name, UTF8String];
        CStr::from_ptr(name).to_string_lossy().into_owned()
    }
}

pub fn get_view_by_id(id: id, name: &'static str) -> Option<id> {
    let nsview: id = if id_is_instance_of(id, "NSWindow") {
        unsafe { msg_send![id, contentView] }
    } else {
        id
    };
    get_view(nsview, &|view| {
        unsafe {
            let identifier: id = msg_send![view, identifier];
            NSString::isEqualToString(identifier, name)
        }
    })
}

pub fn get_view<F>(nsview: id, predicate: &F) -> Option<id> where F: Fn(id) -> bool {
    if predicate(nsview) {
        return Some(nsview);
    }
    unsafe {
        let subviews: id = msg_send![nsview, subviews];
        let count: NSInteger = msg_send![subviews, count];
        for i in 0..count {
            let view: id = msg_send![subviews, objectAtIndex:i];
            if let Some(view) = get_view(view, predicate) {
                return Some(view)
            }
        }
        return None
    }
}


pub fn get_state<'a>() -> &'a AppState {
    unsafe {
        let delegate: id = msg_send![NSApp(), delegate];
        let ivar: *const c_void = *(&*delegate).get_ivar("state");
        &*(ivar as *const AppState)
    }
}

pub fn get_mut_state<'a>() -> &'a mut AppState {
    unsafe {
        let delegate: id = msg_send![NSApp(), delegate];
        let ivar: *mut c_void = *(&*delegate).get_ivar("state");
        &mut *(ivar as *mut AppState)
    }
}
