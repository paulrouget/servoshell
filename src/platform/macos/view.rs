extern crate core_foundation;
extern crate cgl;
extern crate gleam;

use view::DrawableGeometry;

use cocoa::appkit;
use cocoa::appkit::*;
use cocoa::foundation::*;
use cocoa::base::*;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use self::cgl::{CGLEnable, kCGLCECrashOnRemovedFunctions};
use self::core_foundation::base::TCFType;
use self::core_foundation::string::CFString;
use self::core_foundation::bundle::{CFBundleGetBundleWithIdentifier, CFBundleGetFunctionPointerForName};
use std::os::raw::c_void;
use std::str::FromStr;
use view::{ElementState, MouseButton, ViewEvent, TouchPhase, MouseScrollDelta};
use super::utils;

pub fn register() {
    let superclass = Class::get("NSView").unwrap();
    let mut class = ClassDecl::new("NSServoView", superclass).unwrap();

    class.add_ivar::<*mut c_void>("event_queue");

    extern fn store_nsevent(this: &Object, _sel: Sel, event: id) {
        unsafe { msg_send![event, retain] }
        utils::get_event_queue(this).push(event);
    }

    extern fn awake_from_nib(this: &mut Object, _sel: Sel) {
        // FIXME: is that the best way to create a raw pointer?
        let event_queue: Vec<id> = Vec::new();
        let event_queue_ptr = Box::into_raw(Box::new(event_queue));
        unsafe {
            this.set_ivar("event_queue", event_queue_ptr as *mut c_void);
        }
    }

    extern fn accept_first_responder(_this: &Object, _sel: Sel) -> BOOL {
        YES
    }

    unsafe {
        class.add_method(sel!(scrollWheel:), store_nsevent as extern fn(&Object, Sel, id));
        class.add_method(sel!(mouseDown:), store_nsevent as extern fn(&Object, Sel, id));
        class.add_method(sel!(mouseUp:), store_nsevent as extern fn(&Object, Sel, id));
        class.add_method(sel!(mouseMoved:), store_nsevent as extern fn(&Object, Sel, id));

        class.add_method(sel!(acceptsFirstResponder), accept_first_responder as extern fn(&Object, Sel) -> BOOL);

        class.add_method(sel!(awakeFromNib), awake_from_nib as extern fn(&mut Object, Sel));
    }

    class.register();
}

pub struct View {
    nsview: id,
    context: id,
}

impl View {

    pub fn new(nsview: id) -> View {
        let context: id = View::init_gl(nsview);
        View {
            nsview: nsview,
            context: context
        }
    }

    pub fn swap_buffers(&self) {
        unsafe {
            msg_send![self.context, flushBuffer];
        }
    }

    pub fn update_drawable(&self) {
        unsafe {
            msg_send![self.context, update];
        }
    }

    pub fn get_geometry(&self) -> DrawableGeometry {
        unsafe {
            let nswindow: id = msg_send![self.nsview, window];
            let frame: NSRect = msg_send![self.nsview, frame];
            let hidpi_factor: CGFloat = msg_send![nswindow, backingScaleFactor];

            // FIXME: this is going to be wrong as soon as the view is not full contentView
            let visible_rect: NSRect = msg_send![nswindow, contentLayoutRect];

            // FIXME: coordinates are flipped
            let bottom = visible_rect.origin.y - frame.origin.y;
            let left = visible_rect.origin.x - frame.origin.x;
            let right = frame.size.width - left - visible_rect.size.width;
            let top = frame.size.height - bottom - visible_rect.size.height;

            DrawableGeometry {
                view_size: (frame.size.width as u32, frame.size.height as u32),
                margins: (top as u32, right as u32, bottom as u32, left as u32),
                position: (0, 0),
                hidpi_factor: hidpi_factor as f32,
            }
        }
    }

    pub fn get_events(&self) -> Vec<ViewEvent> {
        let nsobject = unsafe { &*self.nsview};
        let event_queue = utils::get_event_queue(nsobject);
        // FIXME: make sure event_queue is empty
        let events = event_queue.drain(..).map(|e| {
            self.nsevent_to_viewevent(e)
        }).filter(|e| {
            e.is_some()
        }).map(|e| {
            e.unwrap()
        }).collect();
        events
    }

    pub fn enter_fullscreen(&self) {
        unsafe {
            msg_send![self.nsview, enterFullScreenMode:nil withOptions:nil];
        }
    }

    pub fn exit_fullscreen(&self) {
        unsafe {
            msg_send![self.nsview, exitFullScreenModeWithOptions:nil];
        }
    }

    fn nsevent_to_viewevent(&self, nsevent: id) -> Option<ViewEvent> {
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
                    Some(ViewEvent::MouseWheel(delta, phase))
                },
                NSMouseMoved => {
                    let nswindow = nsevent.window();
                    let window_point = nsevent.locationInWindow();
                    let view_point = self.nsview.convertPoint_fromView_(window_point, nil);
                    let frame = NSView::frame(self.nsview);
                    let hidpi_factor: CGFloat = msg_send![nswindow, backingScaleFactor];
                    let hidpi_factor = hidpi_factor as f32;
                    let x = (hidpi_factor * view_point.x as f32) as i32;
                    let y = (hidpi_factor * (frame.size.height - view_point.y) as f32) as i32;
                    Some(ViewEvent::MouseMoved(x, y))
                }

                NSLeftMouseDown => { Some(ViewEvent::MouseInput(ElementState::Pressed, MouseButton::Left)) },
                NSLeftMouseUp => { Some(ViewEvent::MouseInput(ElementState::Released, MouseButton::Left)) },
                NSRightMouseDown => { Some(ViewEvent::MouseInput(ElementState::Pressed, MouseButton::Right)) },
                NSRightMouseUp => { Some(ViewEvent::MouseInput(ElementState::Released, MouseButton::Right)) },
                NSOtherMouseDown => { Some(ViewEvent::MouseInput(ElementState::Pressed, MouseButton::Middle)) },
                NSOtherMouseUp => { Some(ViewEvent::MouseInput(ElementState::Released, MouseButton::Middle)) },

                _ => None
            }
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
}


