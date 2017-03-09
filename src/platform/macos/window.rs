use cocoa::appkit::*;
use cocoa::foundation::*;
use cocoa::base::*;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use std::ffi::CStr;
use std::os::raw::c_void;
use super::utils;
use window::WindowEvent;
use std::collections::HashMap;
use commands::{CommandState, WindowCommand};
use libc;
use servo::ServoCursor;

// FIXME: this is ugly. Also, we are duplicating the list
// of Selector (see the add_method list)
fn action_to_command(action: Sel) -> Option<WindowCommand> {
    if action == sel!(shellReload:) {
        Some(WindowCommand::Reload)
    } else if action == sel!(shellStop:) {
        Some(WindowCommand::Stop)
    } else if action == sel!(shellNavigateBack:) {
        Some(WindowCommand::NavigateBack)
    } else if action == sel!(shellNavigateForward:) {
        Some(WindowCommand::NavigateForward)
    } else if action == sel!(shellOpenLocation:) {
        Some(WindowCommand::OpenLocation)
    } else if action == sel!(shellZoomIn:) {
        Some(WindowCommand::ZoomIn)
    } else if action == sel!(shellZoomOut:) {
        Some(WindowCommand::ZoomOut)
    } else if action == sel!(shellZoomToActualSize:) {
        Some(WindowCommand::ZoomToActualSize)
    } else if action == sel!(shellOpenInDefaultBrowser:) {
        Some(WindowCommand::OpenInDefaultBrowser)
    } else {
        None
    }
}


pub fn register() {

    /* NSWindow subclass */ {

        let superclass = Class::get("NSWindow").unwrap();
        let mut class = ClassDecl::new("NSShellWindow", superclass).unwrap();
        class.add_ivar::<*mut c_void>("event_queue");

        extern fn toggle_tabbar(this: &Object, _sel: Sel, sender: id) {
            unsafe {
                msg_send![super(this, Class::get("NSWindow").unwrap()), toggleTabBar:sender];
            }
            utils::get_event_queue(this).push(WindowEvent::GeometryDidChange);
        }

        extern fn toggle_toolbar(this: &Object, _sel: Sel, sender: id) {
            unsafe {
                msg_send![super(this, Class::get("NSWindow").unwrap()), toggleToolbarShown:sender];
            }
            utils::get_event_queue(this).push(WindowEvent::GeometryDidChange);
        }

        extern fn event_loop_rised(this: &Object, _sel: Sel) {
            utils::get_event_queue(this).push(WindowEvent::EventLoopRised);
        }

        extern fn awake_from_nib(this: &mut Object, _sel: Sel) {
            let event_queue: Vec<WindowEvent> = Vec::new();
            // FIXME: is that the best way to create a raw pointer?
            let event_queue_ptr = Box::into_raw(Box::new(event_queue));
            unsafe {
                this.set_ivar("event_queue", event_queue_ptr as *mut c_void);
            }
        }

        unsafe {
            class.add_method(sel!(toggleTabBar:), toggle_tabbar as extern fn(&Object, Sel, id));
            class.add_method(sel!(toggleToolbarShown:), toggle_toolbar as extern fn(&Object, Sel, id));
            class.add_method(sel!(eventLoopRised), event_loop_rised as extern fn(&Object, Sel));
            class.add_method(sel!(awakeFromNib), awake_from_nib as extern fn(&mut Object, Sel));
        }

        class.register();
    }

    /* NSWindowDelegate */ {

        let superclass = Class::get("NSObject").unwrap();
        let mut class = ClassDecl::new("NSShellWindowDelegate", superclass).unwrap();
        class.add_ivar::<*mut c_void>("event_queue");
        class.add_ivar::<*mut c_void>("command_states");

        // FIXME: Don't use strings. And maybe use a map to avoid the duplicate code with add_method.
        extern fn record_notification(this: &Object, _sel: Sel, notification: id) {
            let event = unsafe {
                let name: id = msg_send![notification, name];
                if NSString::isEqualToString(name, "NSWindowDidResizeNotification") {
                    Some(WindowEvent::GeometryDidChange)
                } else if NSString::isEqualToString(name, "NSWindowDidEnterFullScreenNotification") {
                    Some(WindowEvent::DidEnterFullScreen)
                } else if NSString::isEqualToString(name, "NSWindowDidExitFullScreenNotification") {
                    Some(WindowEvent::DidExitFullScreen)
                } else if NSString::isEqualToString(name, "NSWindowWillCloseNotification") {
                    Some(WindowEvent::WillClose)
                } else {
                    None
                }
            };
            utils::get_event_queue(this).push(event.unwrap());
        }

        extern fn validate_ui(this: &Object, _sel: Sel, item: id) -> BOOL {
            unsafe {
                let action: Sel = msg_send![item, action];
                msg_send![this, validateAction:action]
            }
        }

        extern fn validate_action(this: &Object, _sel: Sel, action: Sel) -> BOOL {
            let map: &mut HashMap<WindowCommand, CommandState> = utils::get_command_states(this);
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
                Some(cmd) => utils::get_event_queue(this).push(WindowEvent::DoCommand(cmd)),
                None => panic!("Unexpected action to record"),
            }
        }

        extern fn submit_user_input(this: &Object, _sel: Sel, item: id) {
            let text = unsafe {
                let text: id = msg_send![item, stringValue];
                let text: *const libc::c_char = msg_send![text, UTF8String];
                CStr::from_ptr(text).to_string_lossy().into_owned()
            };
            let cmd = WindowCommand::Load(text);
            utils::get_event_queue(this).push(WindowEvent::DoCommand(cmd));
        }

        extern fn navigate(this: &Object, _sel: Sel, item: id) {
            let idx: NSInteger = unsafe { msg_send![item, selectedSegment] };
            let cmd = if idx == 0 {
                WindowCommand::NavigateBack
            } else {
                WindowCommand::NavigateForward
            };
            utils::get_event_queue(this).push(WindowEvent::DoCommand(cmd));
        }

        extern fn zoom(this: &Object, _sel: Sel, item: id) {
            let idx: NSInteger = unsafe { msg_send![item, selectedSegment] };
            let cmd = if idx == 0 {
                WindowCommand::ZoomOut
            } else  if idx == 1 {
                WindowCommand::ZoomToActualSize
            } else {
                WindowCommand::ZoomIn
            };
            utils::get_event_queue(this).push(WindowEvent::DoCommand(cmd));
        }

        unsafe {
            class.add_method(sel!(windowDidResize:), record_notification as extern fn(&Object, Sel, id));
            class.add_method(sel!(windowDidEnterFullScreen:), record_notification as extern fn(&Object, Sel, id));
            class.add_method(sel!(windowDidExitFullScreen:), record_notification as extern fn(&Object, Sel, id));
            class.add_method(sel!(windowWillClose:), record_notification as extern fn(&Object, Sel, id));

            class.add_method(sel!(shellStop:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellReload:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellOpenLocation:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellZoomIn:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellZoomOut:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellZoomToActualSize:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellNavigateBack:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellNavigateForward:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellOpenInDefaultBrowser:), record_command as extern fn(&Object, Sel, id));

            class.add_method(sel!(shellSubmitUserInput:), submit_user_input as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellNavigate:), navigate as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellZoom:), zoom as extern fn(&Object, Sel, id));

            class.add_method(sel!(validateAction:), validate_action as extern fn(&Object, Sel, Sel) -> BOOL);
            class.add_method(sel!(validateUserInterfaceItem:), validate_ui as extern fn(&Object, Sel, id) -> BOOL);
        }

        class.register();
    }
}


pub struct Window {
    nswindow: id,
}

impl Window {
    pub fn new(nswindow: id) -> Window {

        let command_states: HashMap<WindowCommand, CommandState> = HashMap::new();
        let command_states_ptr = Box::into_raw(Box::new(command_states));

        unsafe {
            // FIXME: release and set delegate to nil
            let delegate: id = msg_send![class("NSShellWindowDelegate"), alloc];
            let event_queue_ptr: *mut c_void = *(&*nswindow).get_ivar("event_queue");
            (*delegate).set_ivar("event_queue", event_queue_ptr);
            (*delegate).set_ivar("command_states", command_states_ptr as *mut c_void);

            msg_send![nswindow, setDelegate:delegate];

            nswindow.setTitleVisibility_(NSWindowTitleVisibility::NSWindowTitleHidden);
            let mask = nswindow.styleMask() as NSUInteger | NSWindowMask::NSFullSizeContentViewWindowMask as NSUInteger;
            nswindow.setStyleMask_(mask);
            nswindow.setAcceptsMouseMovedEvents_(YES);

        }

        Window {
            nswindow: nswindow,
        }
    }

    pub fn get_events(&self) -> Vec<WindowEvent> {
        let nsobject = unsafe { &*self.nswindow};
        utils::get_event_queue(nsobject).drain(..).collect()
    }

    pub fn set_command_state(&self, cmd: WindowCommand, state: CommandState) {
        let nsobject = unsafe {
            let delegate: id = msg_send![self.nswindow, delegate];
            &*delegate
        };
        let command_states = utils::get_command_states(nsobject);
        command_states.insert(cmd, state);
    }

    pub fn set_url(&self, url: &str) {
        // FIXME: can't get NSWindow::representedURL to work
        unsafe {
            let item = self.get_toolbar_item("urlbar").unwrap();
            let urlbar: id = msg_send![item, view];
            let string = NSString::alloc(nil).init_str(url);
            msg_send![urlbar, setStringValue:string];
        }
    }

    pub fn focus_urlbar(&self) {
        unsafe {
            let item = self.get_toolbar_item("urlbar").unwrap();
            let urlbar: id = msg_send![item, view];
            msg_send![urlbar, becomeFirstResponder];
        }
    }

    fn get_toolbar_item(&self, identifier: &str) -> Option<id> {
        unsafe {
            let toolbar: id = msg_send![self.nswindow, toolbar];
            let items: id = msg_send![toolbar, items];
            let count: NSInteger = msg_send![items, count];
            for i in 0..count {
                let item: id = msg_send![items, objectAtIndex:i];
                let item_identifier: id = msg_send![item, itemIdentifier];
                if NSString::isEqualToString(item_identifier, identifier) {
                    return Some(item);
                }
            }
            None
        }
    }

    pub fn set_title(&self, title: &str) {
        unsafe {
            let title = NSString::alloc(nil).init_str(title);
            msg_send![self.nswindow, setTitle:title]
        }
    }

    pub fn create_eventloop_riser(&self) -> EventLoopRiser {
        let window_number: NSInteger = unsafe {
            msg_send![self.nswindow, windowNumber]
        };
        EventLoopRiser {
            window_number: window_number,
        }
    }

    // From winit
    pub fn set_cursor(&self, cursor: ServoCursor) {
        let cursor_name = match cursor {
            ServoCursor::Default => "arrowCursor",
            ServoCursor::Pointer => "pointingHandCursor",
            ServoCursor::ContextMenu => "contextualMenuCursor",
            ServoCursor::Crosshair => "crosshairCursor",
            ServoCursor::Text => "IBeamCursor",
            ServoCursor::VerticalText => "IBeamCursorForVerticalLayout",
            ServoCursor::Alias => "dragLinkCursor",
            ServoCursor::Copy => "dragCopyCursor",
            ServoCursor::NoDrop => "operationNotAllowedCursor",
            ServoCursor::NotAllowed => "operationNotAllowedCursor",
            ServoCursor::Grab => "closedHandCursor",
            ServoCursor::Grabbing => "closedHandCursor",
            ServoCursor::EResize => "resizeRightCursor",
            ServoCursor::NResize => "resizeUpCursor",
            ServoCursor::SResize => "resizeDownCursor",
            ServoCursor::WResize => "resizeLeftCursor",
            ServoCursor::EwResize => "resizeLeftRightCursor",
            ServoCursor::NsResize => "resizeUpDownCursor",
            ServoCursor::ColResize => "resizeLeftRightCursor",
            ServoCursor::RowResize => "resizeUpDownCursor",
            ServoCursor::None |
            ServoCursor::Cell |
            ServoCursor::Move |
            ServoCursor::NeResize |
            ServoCursor::NwResize |
            ServoCursor::SeResize |
            ServoCursor::SwResize |
            ServoCursor::NeswResize |
            ServoCursor::NwseResize |
            ServoCursor::AllScroll |
            ServoCursor::ZoomIn |
            ServoCursor::ZoomOut |
            ServoCursor::Wait |
            ServoCursor::Progress |
            ServoCursor::Help => "arrowServoCursor"
        };
        let sel = Sel::register(cursor_name);
        let cls = Class::get("NSCursor").unwrap();
        unsafe {
            use objc::Message;
            let cursor: id = cls.send_message(sel, ()).unwrap();
            let _: () = msg_send![cursor, set];
        }
    }
}

pub struct EventLoopRiser {
    window_number: NSInteger,
}

impl EventLoopRiser {
    pub fn rise(&self) {
        unsafe {
            let pool = NSAutoreleasePool::new(nil);
            let event: id = msg_send![class("NSEvent"),
                    otherEventWithType:NSApplicationDefined
                    location:NSPoint::new(0.0, 0.0)
                    modifierFlags:NSEventModifierFlags::empty()
                    timestamp:0.0
                    windowNumber:self.window_number
                    context:nil
                    subtype:NSEventSubtype::NSApplicationActivatedEventType
                    data1:0
                    data2:0];
            msg_send![event, retain];
            msg_send![NSApp(), postEvent:event atStart:NO];
            NSAutoreleasePool::drain(pool);
        }
    }
    pub fn clone(&self) -> EventLoopRiser {
        EventLoopRiser {
            window_number: self.window_number,
        }
    }
}
