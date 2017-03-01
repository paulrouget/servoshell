extern crate core_foundation;
extern crate cgl;
extern crate gleam;

use cocoa::appkit;
use cocoa::appkit::*;
use cocoa::foundation::*;
use cocoa::base::*;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use rand::Rng;
use rand::os::OsRng;
use self::cgl::{CGLEnable, kCGLCECrashOnRemovedFunctions};
use self::core_foundation::base::TCFType;
use self::core_foundation::string::CFString;
use self::core_foundation::bundle::{CFBundleGetBundleWithIdentifier, CFBundleGetFunctionPointerForName};
use std::fmt;
use std::os::raw::c_void;
use std::str::FromStr;
use std::sync::{Once, ONCE_INIT};
use view_events::{ViewEvent, TouchPhase, MouseScrollDelta};

static INIT: Once = ONCE_INIT;

pub fn register_nsservoview() {
    unsafe {
        INIT.call_once(|| {

            let superclass = Class::get("NSView").unwrap();
            let mut servoview_class = ClassDecl::new("NSServoView", superclass).unwrap();

            servoview_class.add_ivar::<*mut c_void>("event_queue");
            servoview_class.add_ivar::<NSInteger>("_tag");

            servoview_class.add_method(sel!(eventloopRised:), store_event as extern fn(&Object, Sel, id));
            servoview_class.add_method(sel!(scrollWheel:), store_event as extern fn(&Object, Sel, id));
            servoview_class.add_method(sel!(mouseDown:), store_event as extern fn(&Object, Sel, id));
            servoview_class.add_method(sel!(mouseUp:), store_event as extern fn(&Object, Sel, id));
            servoview_class.add_method(sel!(mouseMoved:), store_event as extern fn(&Object, Sel, id));
            extern fn store_event(this: &Object, _sel: Sel, event: id) {
                unsafe {
                    let event_queue: &mut Vec<id> = {
                        let ivar: *mut c_void = *this.get_ivar("event_queue");
                        &mut *(ivar as *mut Vec<id>)
                    };
                    msg_send![event, retain];
                    event_queue.push(event);
                }
            }

            servoview_class.add_method(sel!(tag), get_tag as extern fn(&Object, Sel) -> NSInteger);
            extern fn get_tag(this: &Object, _sel: Sel) -> NSInteger {
                unsafe { *this.get_ivar("_tag") }
            }


            servoview_class.add_method(sel!(awakeFromNib), awake_from_nib as extern fn(&mut Object, Sel));
            extern fn awake_from_nib(this: &mut Object, _sel: Sel) {
                let event_queue: Vec<id> = Vec::new();
                // FIXME: is that the best way to create a raw pointer?
                let event_queue_ptr = Box::into_raw(Box::new(event_queue));
                unsafe {
                    this.set_ivar("event_queue", event_queue_ptr as *mut c_void);

                    // FIXME: shouldn't that be shared?
                    let mut rand = OsRng::new().unwrap();
                    let tag: NSInteger = rand.gen();
                    this.set_ivar("_tag", tag);
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
        let context: id = ServoView::init_gl(nsview);
        ServoView {
            nsview: nsview,
            context: context
        }
    }

    fn init_gl(nsview: id) -> id {
        let ctx = unsafe {
            nsview.setWantsBestResolutionOpenGLSurface_(YES);
            let attributes = vec![NSOpenGLPFADoubleBuffer as u32,
                                  NSOpenGLPFAClosestPolicy as u32,
                                  NSOpenGLPFAColorSize as u32,
                                  32,
                                  NSOpenGLPFAAlphaSize as u32,
                                  8,
                                  NSOpenGLPFADepthSize as u32,
                                  24,
                                  NSOpenGLPFAStencilSize as u32,
                                  8,
                                  NSOpenGLPFAOpenGLProfile as u32,
                                  NSOpenGLProfileVersion3_2Core as u32,
                                  0];

            let pixelformat = NSOpenGLPixelFormat::alloc(nil).initWithAttributes_(&attributes);
            let ctx: id = NSOpenGLContext::alloc(nil).initWithFormat_shareContext_(pixelformat, nil);
            msg_send![ctx, setView: nsview];
            let value = 1;
            ctx.setValues_forParameter_(&value, NSOpenGLContextParameter::NSOpenGLCPSwapInterval);
            CGLEnable(ctx.CGLContextObj() as *mut _, kCGLCECrashOnRemovedFunctions);
            ctx
        };

        unsafe {
            msg_send![ctx, update];
            msg_send![ctx, makeCurrentContext];
        };

        gleam::gl::load_with(|addr| {
            let symbol_name: CFString = FromStr::from_str(addr).unwrap();
            let framework_name: CFString = FromStr::from_str("com.apple.opengl").unwrap();
            let framework = unsafe {
                CFBundleGetBundleWithIdentifier(framework_name.as_concrete_TypeRef())
            };
            let symbol = unsafe {
                CFBundleGetFunctionPointerForName(framework, symbol_name.as_concrete_TypeRef())
            };
            symbol as *const c_void
        });

        gleam::gl::clear_color(1.0, 0.0, 0.0, 1.0);
        gleam::gl::clear(gleam::gl::COLOR_BUFFER_BIT);
        gleam::gl::finish();

        ctx
    }

    pub fn swap_buffers(&self) {
        unsafe {
            msg_send![self.context, flushBuffer];
        }
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
        let event_queue: &mut Vec<id> = unsafe {
            let ivar: *mut c_void = *(&*self.nsview).get_ivar("event_queue");
            &mut *(ivar as *mut Vec<id>)
        };
        let events = event_queue.into_iter().map(|e| {
            self.nsevent_to_viewevent(e)
        }).collect();
        event_queue.clear();
        events
    }

    fn nsevent_to_viewevent(&self, nsevent: &id) -> ViewEvent {
        unsafe {
            let event_type = nsevent.eventType();
            match event_type {
                NSScrollWheel => {
                    // Stolen from winit
                    use self::MouseScrollDelta::{LineDelta, PixelDelta};
                    let nswindow: id = nsevent.window();
                    let hidpi_factor: CGFloat = msg_send![nswindow, backingScaleFactor];
                    let hidpi_factor = hidpi_factor as f32;
                    let delta = if nsevent.hasPreciseScrollingDeltas() == YES {
                        PixelDelta(hidpi_factor * nsevent.scrollingDeltaX() as f32,
                                   hidpi_factor * nsevent.scrollingDeltaY() as f32)
                    } else {
                        LineDelta(hidpi_factor * nsevent.scrollingDeltaX() as f32,
                                  hidpi_factor * nsevent.scrollingDeltaY() as f32)
                    };
                    let phase = match nsevent.phase() {
                        appkit::NSEventPhaseMayBegin | appkit::NSEventPhaseBegan => TouchPhase::Started,
                        appkit::NSEventPhaseEnded => TouchPhase::Ended,
                        _ => TouchPhase::Moved,
                    };
                    ViewEvent::MouseWheel(delta, phase)
                },
                NSApplicationDefined => match unsafe {nsevent.subtype()} {
                    NSEventSubtype::NSApplicationActivatedEventType => {
                        ViewEvent::Refresh
                    },
                    _ => ViewEvent::Awakened // FIXME: makes no sense
                },
                _ => ViewEvent::Awakened // FIXME: makes no sense
            }
        }
    }

    pub fn create_eventloop_riser(&self) -> EventLoopRiser {
        let (view_tag, window_number) = unsafe {
            let view_tag: NSInteger = msg_send![self.nsview, tag];
            let window: id = msg_send![self.nsview, window];
            let window_number: NSInteger = msg_send![window, windowNumber];
            (view_tag, window_number)
        };
        EventLoopRiser {
            window_number: window_number,
            view_tag: view_tag,
        }
    }

    // pub fn focus() {
    // }

    // pub fn unfocus() {
    // }

    // pub fn is_focused() -> bool {
    //     false
    // }

    // pub fn go_fullscreen() {
    // }

    // pub fn leave_fullscreen() {
    // }

    // pub fn is_fullscreen() -> bool {
    //     false
    // }
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
