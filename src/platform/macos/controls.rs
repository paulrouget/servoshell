use cocoa::base::*;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use std::os::raw::c_void;
use super::utils;
use controls::ControlEvent;
use std::collections::HashMap;

// FIXME: implement Hash for Sel instead of using a str
// FIXME: can we use macro instead of this hashmap (which is instanciate at 2 different locations)
fn map_action_to_event() -> HashMap<&'static str, ControlEvent> {
    let mut map = HashMap::new();
    map.insert("reload:", ControlEvent::Reload);
    map.insert("stop:", ControlEvent::Stop);
    map.insert("goBack:", ControlEvent::GoBack);
    map.insert("goForward:", ControlEvent::GoForward);
    map.insert("zoomIn:", ControlEvent::ZoomIn);
    map.insert("zoomOut:", ControlEvent::ZoomOut);
    map.insert("zoomImageToActualSize:", ControlEvent::ZoomToActualSize);
    map
}

pub fn register() {
    let superclass = Class::get("NSResponder").unwrap();
    let mut class = ClassDecl::new("NSShellResponder", superclass).unwrap();
    class.add_ivar::<*mut c_void>("event_queue");
    class.add_ivar::<*mut c_void>("command_states");
    class.add_ivar::<*mut c_void>("action_to_event");

    for action in map_action_to_event().keys() {
        unsafe {
            class.add_method(selector(action), record_action as extern fn(&Object, Sel, id));
        }
    }
    unsafe {
        class.add_method(sel!(validateUserInterfaceItem:), validate_ui as extern fn(&Object, Sel, id) -> BOOL);
    }

    extern fn record_action(this: &Object, _sel: Sel, item: id) {
        let action: Sel = unsafe {msg_send![item, action]};
        let action = action.name();
        let action_to_event: &mut HashMap<&str, ControlEvent> = utils::get_ivar(this, "action_to_event");
        let event = action_to_event.get(action).unwrap();
        utils::get_event_queue(this).push(event);
    }

    extern fn validate_ui(this: &Object, _sel: Sel, item: id) -> BOOL {
        let map: &mut HashMap<ControlEvent, bool> = utils::get_command_states(this);
        let action: Sel = unsafe {msg_send![item, action]};
        let action = action.name();
        let action_to_event: &mut HashMap<&str, ControlEvent> = utils::get_ivar(this, "action_to_event");
        let event = action_to_event.get(action).unwrap();
        match map.get(&event) {
            Some(enabled) if *enabled => YES,
            _ => NO
        }
    }

    class.register();
}

pub struct Controls {
    nsresponder: id,
}

impl Controls {

    pub fn new() -> Controls {

        let command_states: HashMap<ControlEvent, bool> = HashMap::new();
        let command_states_ptr = Box::into_raw(Box::new(command_states));

        let action_to_event = map_action_to_event();
        let action_to_event_ptr = Box::into_raw(Box::new(action_to_event));

        let event_queue: Vec<ControlEvent> = Vec::new();
        let event_queue_ptr = Box::into_raw(Box::new(event_queue));

        let nsresponder = unsafe {
            let nsresponder: id = msg_send![class("NSShellResponder"), alloc];
            (*nsresponder).set_ivar("command_states", command_states_ptr as *mut c_void);
            (*nsresponder).set_ivar("action_to_event", action_to_event_ptr as *mut c_void);
            (*nsresponder).set_ivar("event_queue", event_queue_ptr as *mut c_void);
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
