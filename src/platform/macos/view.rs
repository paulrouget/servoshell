/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate core_foundation;
extern crate cgl;

use view::DrawableGeometry;

use cocoa::appkit;
use cocoa::appkit::*;
use cocoa::foundation::*;
use cocoa::base::*;
use gleam::gl;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use std::rc::Rc;
use self::cgl::{CGLEnable, kCGLCECrashOnRemovedFunctions};
use self::core_foundation::base::TCFType;
use self::core_foundation::string::CFString;
use self::core_foundation::bundle::{CFBundleGetBundleWithIdentifier, CFBundleGetFunctionPointerForName};
use std::os::raw::c_void;
use std::str::FromStr;
use view::{ElementState, MouseButton, ViewEvent, TouchPhase, MouseScrollDelta};
use super::get_state;
use super::utils;

pub fn register() {
    let superclass = Class::get("NSView").unwrap();
    let mut class = ClassDecl::new("NSServoView", superclass).unwrap();

    class.add_ivar::<*mut c_void>("event_queue");
    class.add_ivar::<*mut c_void>("live_resize_callback");

    extern fn store_nsevent(this: &Object, _sel: Sel, nsevent: id) {
        let event = {
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
                        let view_point: NSPoint = msg_send![this, convertPoint:window_point fromView:nil];
                        let frame: NSRect = msg_send![this, frame];
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

        };

        if let Some(event) = event {
            utils::get_event_queue(this).push(event);
        }
    }

    extern fn awake_from_nib(this: &mut Object, _sel: Sel) {
        // FIXME: is that the best way to create a raw pointer?
        let event_queue: Vec<ViewEvent> = Vec::new();
        let event_queue_ptr = Box::into_raw(Box::new(event_queue));
        unsafe {
            this.set_ivar("event_queue", event_queue_ptr as *mut c_void);
        }
    }

    extern fn accept_first_responder(_this: &Object, _sel: Sel) -> BOOL {
        YES
    }

    extern fn set_frame_size(this: &Object, _sel: Sel, size: NSSize) {
        unsafe {
            msg_send![super(this, Class::get("NSView").unwrap()), setFrameSize:size];
            utils::get_event_queue(this).push(ViewEvent::GeometryDidChange);
            let live_resize: BOOL = msg_send![this, inLiveResize];
            if live_resize == YES {
                let ivar: *mut c_void = *this.get_ivar("live_resize_callback");
                let callback: &Fn() = *(ivar as *mut &Fn());
                callback();
            }
        }
    }

    unsafe {
        class.add_method(sel!(scrollWheel:), store_nsevent as extern fn(&Object, Sel, id));
        class.add_method(sel!(mouseDown:), store_nsevent as extern fn(&Object, Sel, id));
        class.add_method(sel!(mouseUp:), store_nsevent as extern fn(&Object, Sel, id));
        class.add_method(sel!(mouseMoved:), store_nsevent as extern fn(&Object, Sel, id));

        class.add_method(sel!(acceptsFirstResponder), accept_first_responder as extern fn(&Object, Sel) -> BOOL);

        class.add_method(sel!(setFrameSize:), set_frame_size as extern fn(&Object, Sel, NSSize));

        class.add_method(sel!(awakeFromNib), awake_from_nib as extern fn(&mut Object, Sel));
    }

    class.register();
}

pub struct View {
    nsview: id,
    context: id,
    gl: Rc<gl::Gl>,
}

impl View {

    pub fn new(nsview: id) -> View {
        let (context, gl) = View::init_gl(nsview);
        View {
            nsview: nsview,
            context: context,
            gl: gl.clone(),
        }
    }

    pub fn gl(&self) -> Rc<gl::Gl> {
        self.gl.clone()
    }

    pub fn set_live_resize_callback<F>(&self, callback: &F) where F: Fn() {
        // FIXME: If I don't specify the type, segfault… why???
        let ptr: *mut &Fn() = Box::into_raw(Box::new(callback));
        unsafe {
            (*self.nsview).set_ivar("live_resize_callback", ptr as *mut c_void);
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
            let content_view: id = msg_send![nswindow, contentView];

            let hidpi_factor: CGFloat = msg_send![nswindow, backingScaleFactor];

            let view_frame: NSRect = msg_send![self.nsview, frame];
            let content_frame: NSRect = msg_send![content_view, frame];
            let visible_rect: NSRect = msg_send![nswindow, contentLayoutRect];

            let tabheight = if get_state().window_states[0].browser_states.len() > 1 {
                // FIXME
                35.0
            } else {
                0.0
            };

            let bottom = 0;
            let top = (content_frame.size.height - visible_rect.size.height + tabheight) as u32;
            let left = 0;
            let right = 0;

            DrawableGeometry {
                view_size: (view_frame.size.width as u32, view_frame.size.height as u32),
                margins: (top, right, bottom, left),
                position: (0, 0),
                hidpi_factor: hidpi_factor as f32,
            }
        }
    }

    pub fn get_events(&self) -> Vec<ViewEvent> {
        // FIXME: we should allow only one GeometryDidChange
        let nsobject = unsafe { &*self.nsview};
        utils::get_event_queue(nsobject).drain(..).collect()
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

    fn init_gl(nsview: id) -> (id, Rc<gl::Gl>) {
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

        let gl = unsafe {
            gl::GlFns::load_with(|addr| {
                let symbol_name: CFString = FromStr::from_str(addr).unwrap();
                let framework_name: CFString = FromStr::from_str("com.apple.opengl").unwrap();
                let framework = CFBundleGetBundleWithIdentifier(framework_name.as_concrete_TypeRef());
                let symbol = CFBundleGetFunctionPointerForName(framework, symbol_name.as_concrete_TypeRef());
                symbol as *const c_void
            })
        };

        gl.clear_color(1.0, 1.0, 1.0, 1.0);
        gl.clear(gl::COLOR_BUFFER_BIT);
        gl.finish();

        (ctx, gl)
    }
}


