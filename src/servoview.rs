use cocoa::appkit::*;
use cocoa::foundation::*;
use cocoa::base::*;

use initgl;

use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};

use std::fmt;
use std::os::raw::c_void;
use std::sync::{Once, ONCE_INIT};


#[derive(Clone)]
pub enum ViewEvent {
    Unknown,
    MouseDown,
}

impl fmt::Debug for ViewEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ViewEvent::Unknown => write!(f, "Unknown"),
            ViewEvent::MouseDown => write!(f, "MouseDown"),
        }
    }
}


static INIT: Once = ONCE_INIT;

pub fn register_nsservoview() {
    unsafe {
        INIT.call_once(|| {
            let superclass = Class::get("NSView").unwrap();
            let mut servoview_class = ClassDecl::new("NSServoView", superclass).unwrap();

            servoview_class.add_ivar::<*mut c_void>("event_queue");

            servoview_class.add_method(sel!(mouseDown:), store_event as extern fn(&Object, Sel, id));
            servoview_class.add_method(sel!(mouseMoved:), store_event as extern fn(&Object, Sel, id));
            extern fn store_event(this: &Object, _sel: Sel, event: id) {
                unsafe {
                    let event_queue: &mut Vec<id> = {
                        let ivar: *mut c_void = *this.get_ivar("event_queue");
                        &mut *(ivar as *mut Vec<id>)
                    };
                    event_queue.push(event);
                }
            }

            servoview_class.add_method(sel!(initWithFrame:), init_with_frame as extern fn(&Object, Sel, id) -> id);
            extern fn init_with_frame(this: &Object, _sel: Sel, frame: id) -> id {
                let event_queue: Vec<id> = Vec::new();
                // FIXME: is that the best way to create a raw pointer?
                let event_queue_ptr = Box::into_raw(Box::new(event_queue));
                unsafe {
                    let superclass = this.class().superclass().unwrap();
                    let view: id = msg_send![super(this, superclass), initWithFrame:frame];
                    (*view).set_ivar("event_queue", event_queue_ptr as *mut c_void);
                    view
                }
            }

            servoview_class.register();
        });
    }
}

pub struct ServoView {
    nsview: id,
    context: id,
}

impl ServoView {
    pub fn new(nsview: id) -> ServoView {
        let context: id = initgl::init(nsview);
        ServoView {
            nsview: nsview,
            context: context
        }
    }

    pub fn swap_buffers(&self) {
        unsafe {
            msg_send![self.context, flushBuffer];
        }
        // FIXME: call servo.handle_event()
    }

    pub fn get_geometry(&self) -> DrawableGeometry {
        unsafe {
            let nswindow: id = msg_send![self.nsview, window];
            let frame: NSRect = msg_send![self.nsview, frame];
            let hidpi_factor: CGFloat = msg_send![nswindow, backingScaleFactor];
            DrawableGeometry {
                inner_size: (frame.size.width as u32, frame.size.height as u32),
                position: (0, 0),
                hidpi_factor: hidpi_factor as f32,
            }
        }
    }

    pub fn get_events(&self) -> Vec<ViewEvent> {
        println!("1");
        let event_queue: &mut Vec<id> = unsafe {
            println!("2");
            let ivar: *mut c_void = *(&*self.nsview).get_ivar("event_queue");
            println!("3");
            &mut *(ivar as *mut Vec<id>)
        };
        println!("4");
        let r = event_queue.into_iter().map(|e| {
            println!("a");
            let x = self.nsevent_to_viewevent(e);
            println!("b");
            x
        }).collect();
        println!("5");

        println!("size of event queue after get_events: {}", event_queue.len());

        r
    }

    fn nsevent_to_viewevent(&self, nsevent: &id) -> ViewEvent {
        let event_type = unsafe {nsevent.eventType()};
        match event_type {
            NSLeftMouseDown => ViewEvent::MouseDown,
            _ => ViewEvent::Unknown
        }
    }

    pub fn create_eventloop_riser(&self) -> EventLoopRiser {
        let window_number: NSInteger = unsafe {
            let window: id = msg_send![self.nsview, window];
            msg_send![window, windowNumber]
        };

        EventLoopRiser {
            window_number: window_number
        }
    }

    pub fn focus() {
    }

    pub fn unfocus() {
    }

    pub fn is_focused() -> bool {
        false
    }

    pub fn go_fullscreen() {
    }

    pub fn leave_fullscreen() {
    }

    pub fn is_fullscreen() -> bool {
        false
    }
}

#[derive(Copy, Clone)]
pub struct DrawableGeometry {
    pub inner_size: (u32, u32),
    pub position: (i32, i32),
    pub hidpi_factor: f32,
}

// Used by Servo to wake up the event loop
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
            window_number: self.window_number
        }
    }
}
