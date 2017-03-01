pub mod app;
mod utils;
mod servo_view;

pub use self::servo_view::ServoView;

use cocoa::appkit::*;
use cocoa::base::*;
use cocoa::foundation::*;

// Used by Servo to wake up the event loop
pub struct EventLoopRiser {
    window_number: NSInteger,
    view_tag: NSInteger,
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
                    data1:self.view_tag
                    data2:0];
            msg_send![event, retain];
            msg_send![NSApp(), postEvent:event atStart:NO];
            NSAutoreleasePool::drain(pool);
        }
    }
    pub fn clone(&self) -> EventLoopRiser {
        EventLoopRiser {
            window_number: self.window_number,
            view_tag: self.view_tag,
        }
    }
}
