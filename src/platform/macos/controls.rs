use cocoa::appkit::*;
use cocoa::foundation::*;
use cocoa::base::*;
use objc::Message;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use std::os::raw::c_void;
use super::utils;
use controls::ControlEvent;
use std::collections::HashMap;

fn register() {
    let superclass = Class::get("NSResponder").unwrap();
    let mut class = ClassDecl::new("NSShellResponder", superclass).unwrap();
    class.add_ivar::<*mut c_void>("event_queue");
    class.add_ivar::<*mut c_void>("command_states");

    fn action_to_event(action: Sel) -> Option<ControlEvent> {
        if action == sel!(reload:) {
            Some(ControlEvent::Reload)
        } else if action == sel!(stop:) {
            Some(ControlEvent::Stop)
        } else {
            None
        }
    }

    extern fn record_action(this: &Object, _sel: Sel, item: id) {
        let action: Sel = unsafe {msg_send![item, action]};
        if let Some(event) = action_to_event(action) {
            utils::get_event_queue(this).push(event);
        }
    }

    extern fn validate_ui(this: &Object, _sel: Sel, item: id) -> BOOL {
        let map: &mut HashMap<ControlEvent, bool> = utils::get_command_states(this);
        let action: Sel = unsafe {msg_send![item, action]};
        match action_to_event(action) {
            Some(event) => {
                match map.get(&event) {
                    Some(enabled) if *enabled => YES,
                    _ => NO
                }
            }
            None => NO
        }
    }

    unsafe {
        class.add_method(sel!(reload:), record_action as extern fn(&Object, Sel, id));
        class.add_method(sel!(stop:), record_action as extern fn(&Object, Sel, id));
        class.add_method(sel!(validateUserInterfaceItem:), validate_ui as extern fn(&Object, Sel, id) -> BOOL);
    }

    class.register();
}

pub struct Controls {
    nsresponder: id,
}

impl Controls {

    pub fn new() -> Controls {
        register();

        let command_states: HashMap<ControlEvent, bool> = HashMap::new();
        let command_states_ptr = Box::into_raw(Box::new(command_states));


        let event_queue: Vec<ControlEvent> = Vec::new();
        let event_queue_ptr = Box::into_raw(Box::new(event_queue));

        let nsresponder = unsafe {
            let nsresponder: id = msg_send![class("NSShellResponder"), alloc];
            (*nsresponder).set_ivar("event_queue", event_queue_ptr as *mut c_void);
            (*nsresponder).set_ivar("command_states", command_states_ptr as *mut c_void);
            nsresponder
        };

        Controls {
            nsresponder: nsresponder
        }
    }

    pub fn get_events(&self) -> Vec<ControlEvent> {
        let nsobject = unsafe { &*self.nsresponder};
        utils::get_event_queue(nsobject).drain(..).collect()
    }

    pub fn get_nsresponder(&self) -> &Object {
        unsafe { &*self.nsresponder}
    }

    pub fn set_command_state(&self, event: ControlEvent, enabled: bool) {
        let nsobject = unsafe { &*self.nsresponder};
        let command_states = utils::get_command_states(nsobject);
        command_states.insert(event, enabled);
    }

}
