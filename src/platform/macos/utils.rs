use cocoa::base::*;
use cocoa::foundation::*;
use objc::runtime::Object;
use std::os::raw::c_void;
use std::collections::HashMap;
use std::ffi::CStr;
use platform;
use libc;

pub fn load_nib(filename: &str) -> Result<Vec<id>, &'static str> {

    let path = platform::get_resources_path().unwrap();
    let path = path.join("nibs").join(filename);
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

pub fn get_command_states<A, B>(obj: &Object) -> &mut HashMap<A, B> {
    get_ivar(obj, "command_states")
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
