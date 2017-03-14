use cocoa::appkit::*;
use cocoa::base::*;
use cocoa::foundation::*;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use std::os::raw::c_void;
use super::window;
use super::utils;
use app::AppEvent;
use std::collections::HashMap;
use commands::{CommandState, AppCommand};

// FIXME: this is ugly. Also, we are duplicating the list
// of Selector (see the add_method list)
fn action_to_command(action: Sel) -> Option<AppCommand> {
    if action == sel!(shellClearHistory:) {
        Some(AppCommand::ClearHistory)
    } else {
        None
    }
}


pub fn register() {
    let superclass = Class::get("NSResponder").unwrap();
    let mut class = ClassDecl::new("NSShellApplicationDelegate", superclass).unwrap();
    class.add_ivar::<*mut c_void>("event_queue");
    class.add_ivar::<*mut c_void>("command_states");

    extern fn did_finish_launching(this: &Object, _sel: Sel, _notification: id) {
        utils::get_event_queue(this).push(AppEvent::DidFinishLaunching)
    }

    extern fn did_change_screen_parameter(this: &Object, _sel: Sel, _notification: id) {
        utils::get_event_queue(this).push(AppEvent::DidChangeScreenParameters)
    }

    extern fn will_terminate(this: &Object, _sel: Sel, _notification: id) {
        utils::get_event_queue(this).push(AppEvent::WillTerminate)
    }

    extern fn validate_ui(this: &Object, _sel: Sel, item: id) -> BOOL {
        let map: &mut HashMap<AppCommand, CommandState> = utils::get_command_states(this);
        let action: Sel = unsafe {msg_send![item, action]};
        match action_to_command(action) {
            Some(event) => {
                match map.get(&event) {
                    Some(&CommandState::Enabled) => YES,
                    Some(&CommandState::Disabled) => NO,
                    None => NO,
                }
            },
            None => panic!("Unexpected action to validate"),
        }
    }

    extern fn record_command(this: &Object, _sel: Sel, item: id) {
        let action: Sel = unsafe {msg_send![item, action]};
        match action_to_command(action) {
            Some(cmd) => utils::get_event_queue(this).push(AppEvent::DoCommand(cmd)),
            None => panic!("Unexpected action to record"),
        }
    }


    unsafe {
        class.add_method(sel!(applicationDidFinishLaunching:), did_finish_launching as extern fn(&Object, Sel, id));
        class.add_method(sel!(applicationDidChangeScreenParameter:), did_change_screen_parameter as extern fn(&Object, Sel, id));
        class.add_method(sel!(applicationWillTerminate:), will_terminate as extern fn(&Object, Sel, id));

        class.add_method(sel!(shellClearHistory:), record_command as extern fn(&Object, Sel, id));

        class.add_method(sel!(validateUserInterfaceItem:), validate_ui as extern fn(&Object, Sel, id) -> BOOL);
    }

    class.register();
}


pub struct App {
    nsapp: id
}

impl App {

    pub fn load() -> Result<App, &'static str> {

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

        let command_states: HashMap<AppCommand, CommandState> = HashMap::new();
        let command_states_ptr = Box::into_raw(Box::new(command_states));

        unsafe {
            let delegate: id = msg_send![class("NSShellApplicationDelegate"), alloc];
            (*delegate).set_ivar("event_queue", event_queue_ptr as *mut c_void);
            (*delegate).set_ivar("command_states", command_states_ptr as *mut c_void);
            msg_send![nsapp, setDelegate:delegate];
        }

        let app = App {nsapp: nsapp};

        Ok(app)
    }

    pub fn get_events(&self) -> Vec<AppEvent> {
        let nsobject = unsafe {
            let delegate: id = msg_send![self.nsapp, delegate];
            &*delegate
        };
        utils::get_event_queue(nsobject).drain(..).collect()
    }

    pub fn set_command_state(&self, cmd: AppCommand, state: CommandState) {
        let nsobject = unsafe {
            let delegate: id = msg_send![self.nsapp, delegate];
            &*delegate
        };
        let command_states = utils::get_command_states(nsobject);
        command_states.insert(cmd, state);
    }

    // Equivalent of NSApp.run()
    pub fn run<F>(&self, mut callback: F) where F: FnMut() {

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

    pub fn create_window(&self) -> Result<window::Window, &'static str> {
        let nswindow = match App::create_native_window() {
            Ok(w) => w,
            Err(msg) => return Err(msg),
        };

        Ok(window::Window::new(nswindow))
    }

    fn create_native_window() -> Result<id, &'static str> {
        let instances = match utils::load_nib("Window.nib") {
            Ok(instances) => instances,
            Err(msg) => return Err(msg),
        };

        let nswindow = instances.into_iter().find(|i| {
            utils::id_is_instance_of(*i, "NSShellWindow")
        });

        let nswindow = match nswindow {
            None => return Err("Couldn't not find NSWindow instance in nib file"),
            Some(id) => id,
        };

        Ok(nswindow)
    }
}
