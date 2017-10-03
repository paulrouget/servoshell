/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate core_foundation;
extern crate cgl;

use cocoa::appkit::*;
use cocoa::appkit;
use cocoa::base::*;
use cocoa::foundation::*;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use self::cgl::{CGLEnable, kCGLCECrashOnRemovedFunctions};
use self::core_foundation::base::TCFType;
use self::core_foundation::bundle::{CFBundleGetBundleWithIdentifier,
                                    CFBundleGetFunctionPointerForName};
use self::core_foundation::string::CFString;
use std::os::raw::c_void;
use std::rc::Rc;
use std::{ffi, str};
use super::utils;
use traits::view::*;

pub fn register() {
    let superclass = Class::get("NSView").unwrap();
    let mut class = ClassDecl::new("NSServoView", superclass).unwrap();

    class.add_ivar::<*mut c_void>("event_queue");
    class.add_ivar::<*mut c_void>("live_resize_callback");

    extern "C" fn store_nsevent(this: &Object, _sel: Sel, nsevent: id) {
        unsafe {
            let event_type = nsevent.eventType();
            match event_type {
                // FIXME: support NSFlagsChanged
                NSKeyDown | NSKeyUp => {
                    let vkey = to_virtual_key_code(NSEvent::keyCode(nsevent));
                    if vkey.is_none() {
                        warn!("Keyboard input without virtual key.");
                        return;
                    }
                    let vkey = vkey.unwrap();
                    let received_c_str = NSString::UTF8String(nsevent.characters());
                    let received_str = ffi::CStr::from_ptr(received_c_str);
                    let c = str::from_utf8(received_str.to_bytes())
                        .unwrap()
                        .chars()
                        .last();
                    let c = c.and_then(|c| if is_printable(vkey) { Some(c) } else { None });
                    let state = match event_type {
                        NSKeyUp => KeyState::Released,
                        NSKeyDown => KeyState::Pressed,
                        _ => unreachable!("Has to be NSKeyDown or NSKeyUp"),
                    };
                    let mods = to_mods(nsevent);
                    let event = ViewEvent::KeyEvent(c, vkey, state, mods);
                    utils::get_event_queue(this).push(event);
                }

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
                        appkit::NSEventPhaseMayBegin |
                        appkit::NSEventPhaseBegan => TouchPhase::Started,
                        appkit::NSEventPhaseEnded => TouchPhase::Ended,
                        _ => TouchPhase::Moved,
                    };
                    let event = ViewEvent::MouseWheel(delta, phase);
                    utils::get_event_queue(this).push(event);
                }
                NSMouseMoved => {
                    let (x, y) = cursor_coordinates_in_view(this, nsevent);
                    let event = ViewEvent::MouseMoved(x, y);
                    utils::get_event_queue(this).push(event);
                }

                NSLeftMouseDown => {
                    let (x, y) = cursor_coordinates_in_view(this, nsevent);
                    utils::get_event_queue(this).push(ViewEvent::MouseInput(ElementState::Pressed,
                                                                            MouseButton::Left,
                                                                            x,
                                                                            y))
                }
                NSLeftMouseUp => {
                    let (x, y) = cursor_coordinates_in_view(this, nsevent);
                    utils::get_event_queue(this).push(ViewEvent::MouseInput(ElementState::Released,
                                                                            MouseButton::Left,
                                                                            x,
                                                                            y))
                }
                NSRightMouseDown => {
                    let (x, y) = cursor_coordinates_in_view(this, nsevent);
                    utils::get_event_queue(this).push(ViewEvent::MouseInput(ElementState::Pressed,
                                                                            MouseButton::Right,
                                                                            x,
                                                                            y))
                }
                NSRightMouseUp => {
                    let (x, y) = cursor_coordinates_in_view(this, nsevent);
                    utils::get_event_queue(this).push(ViewEvent::MouseInput(ElementState::Released,
                                                                            MouseButton::Right,
                                                                            x,
                                                                            y))
                }
                NSOtherMouseDown => {
                    let (x, y) = cursor_coordinates_in_view(this, nsevent);
                    utils::get_event_queue(this).push(ViewEvent::MouseInput(ElementState::Pressed,
                                                                            MouseButton::Middle,
                                                                            x,
                                                                            y))
                }
                NSOtherMouseUp => {
                    let (x, y) = cursor_coordinates_in_view(this, nsevent);
                    utils::get_event_queue(this).push(ViewEvent::MouseInput(ElementState::Released,
                                                                            MouseButton::Middle,
                                                                            x,
                                                                            y))
                }

                _ => {}
            }
        }
    }

    extern "C" fn awake_from_nib(this: &mut Object, _sel: Sel) {
        let event_queue: Vec<ViewEvent> = Vec::new();
        let event_queue_ptr = Box::into_raw(Box::new(event_queue));
        unsafe {
            this.set_ivar("event_queue", event_queue_ptr as *mut c_void);
        }
    }

    extern "C" fn accept_first_responder(_this: &Object, _sel: Sel) -> BOOL {
        YES
    }

    extern "C" fn set_frame_size(this: &Object, _sel: Sel, size: NSSize) {
        unsafe {
            msg_send![super(this, Class::get("NSView").unwrap()),
                      setFrameSize: size];
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
        class.add_method(sel!(scrollWheel:),
                         store_nsevent as extern "C" fn(&Object, Sel, id));
        class.add_method(sel!(mouseDown:),
                         store_nsevent as extern "C" fn(&Object, Sel, id));
        class.add_method(sel!(mouseUp:),
                         store_nsevent as extern "C" fn(&Object, Sel, id));
        class.add_method(sel!(mouseMoved:),
                         store_nsevent as extern "C" fn(&Object, Sel, id));
        class.add_method(sel!(keyDown:),
                         store_nsevent as extern "C" fn(&Object, Sel, id));
        class.add_method(sel!(keyUp:),
                         store_nsevent as extern "C" fn(&Object, Sel, id));

        class.add_method(sel!(acceptsFirstResponder),
                         accept_first_responder as extern "C" fn(&Object, Sel) -> BOOL);

        class.add_method(sel!(setFrameSize:),
                         set_frame_size as extern "C" fn(&Object, Sel, NSSize));

        class.add_method(sel!(awakeFromNib),
                         awake_from_nib as extern "C" fn(&mut Object, Sel));
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
            let ctx: id =
                NSOpenGLContext::alloc(nil).initWithFormat_shareContext_(pixelformat, nil);
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
                let symbol_name: CFString = str::FromStr::from_str(addr).unwrap();
                let framework_name: CFString = str::FromStr::from_str("com.apple.opengl").unwrap();
                let framework = CFBundleGetBundleWithIdentifier(framework_name
                                                                    .as_concrete_TypeRef());
                let symbol = CFBundleGetFunctionPointerForName(framework,
                                                               symbol_name.as_concrete_TypeRef());
                symbol as *const c_void
            })
        };

        gl.clear_color(1.0, 1.0, 1.0, 1.0);
        gl.clear(gl::COLOR_BUFFER_BIT);
        gl.finish();

        (ctx, gl)
    }
}

impl ViewMethods for View {
    fn gl(&self) -> Rc<gl::Gl> {
        self.gl.clone()
    }

    fn set_live_resize_callback(&self, callback: &FnMut()) {
        // FIXME: If I don't specify the type, segfault… why???
        let ptr: *mut &FnMut() = Box::into_raw(Box::new(callback));
        unsafe {
            (*self.nsview).set_ivar("live_resize_callback", ptr as *mut c_void);
        }
    }

    fn swap_buffers(&self) {
        unsafe {
            msg_send![self.context, flushBuffer];
        }
    }

    fn update_drawable(&self) {
        unsafe {
            msg_send![self.context, update];
        }
    }

    fn get_geometry(&self) -> DrawableGeometry {
        unsafe {
            let nswindow: id = msg_send![self.nsview, window];
            let content_view: id = msg_send![nswindow, contentView];

            let hidpi_factor: CGFloat = msg_send![nswindow, backingScaleFactor];

            let view_frame: NSRect = msg_send![self.nsview, frame];
            let content_frame: NSRect = msg_send![content_view, frame];
            let visible_rect: NSRect = msg_send![nswindow, contentLayoutRect];

            let tabview = utils::get_view_by_id(nswindow, "tabview").expect("Can't find tabview");
            let count: usize = msg_send![tabview, numberOfTabViewItems];
            let tabheight = if count > 1 {
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

    fn get_events(&self) -> Vec<ViewEvent> {
        // FIXME: we should allow only one GeometryDidChange
        let nsobject = unsafe { &*self.nsview };
        utils::get_event_queue(nsobject).drain(..).collect()
    }

    // FIXME: should be controlled by state
    fn enter_fullscreen(&self) {
        unsafe {
            msg_send![self.nsview, enterFullScreenMode:nil withOptions:nil];
        }
    }

    // FIXME: should be controlled by state
    fn exit_fullscreen(&self) {
        unsafe {
            msg_send![self.nsview, exitFullScreenModeWithOptions: nil];
        }
    }
}


// FIXME: move to utils
fn to_virtual_key_code(code: u16) -> Option<Key> {
    match code {
        0x00 => Some(Key::A),
        0x01 => Some(Key::S),
        0x02 => Some(Key::D),
        0x03 => Some(Key::F),
        0x04 => Some(Key::H),
        0x05 => Some(Key::G),
        0x06 => Some(Key::Z),
        0x07 => Some(Key::X),
        0x08 => Some(Key::C),
        0x09 => Some(Key::V),
        //0x0a => World 1,
        0x0b => Some(Key::B),
        0x0c => Some(Key::Q),
        0x0d => Some(Key::W),
        0x0e => Some(Key::E),
        0x0f => Some(Key::R),
        0x10 => Some(Key::Y),
        0x11 => Some(Key::T),
        0x12 => Some(Key::Num1),
        0x13 => Some(Key::Num2),
        0x14 => Some(Key::Num3),
        0x15 => Some(Key::Num4),
        0x16 => Some(Key::Num6),
        0x17 => Some(Key::Num5),
        0x18 => Some(Key::Equal),
        0x19 => Some(Key::Num9),
        0x1a => Some(Key::Num7),
        0x1b => Some(Key::Minus),
        0x1c => Some(Key::Num8),
        0x1d => Some(Key::Num0),
        0x1e => Some(Key::RightBracket),
        0x1f => Some(Key::O),
        0x20 => Some(Key::U),
        0x21 => Some(Key::LeftBracket),
        0x22 => Some(Key::I),
        0x23 => Some(Key::P),
        0x24 => Some(Key::Enter),
        0x25 => Some(Key::L),
        0x26 => Some(Key::J),
        0x27 => Some(Key::Apostrophe),
        0x28 => Some(Key::K),
        0x29 => Some(Key::Semicolon),
        0x2a => Some(Key::Backslash),
        0x2b => Some(Key::Comma),
        0x2c => Some(Key::Slash),
        0x2d => Some(Key::N),
        0x2e => Some(Key::M),
        0x2f => Some(Key::Period),
        0x30 => Some(Key::Tab),
        0x31 => Some(Key::Space),
        0x32 => Some(Key::GraveAccent),
        0x33 => Some(Key::Backspace),
        //0x34 => unkown,
        0x35 => Some(Key::Escape),
        0x36 => Some(Key::RightSuper),
        0x37 => Some(Key::LeftSuper),
        0x38 => Some(Key::LeftShift),
        //0x39 => Caps lock,
        //0x3a => Left alt,
        0x3b => Some(Key::LeftControl),
        0x3c => Some(Key::RightShift),
        //0x3d => Right alt,
        0x3e => Some(Key::RightControl),
        //0x3f => Fn key,
        //0x40 => F17 Key,
        //0x41 => Some(Key::Decimal),
        //0x42 -> unkown,
        // 0x43 => Some(Key::Multiply),
        //0x44 => unkown,
        // 0x45 => Some(Key::Add),
        //0x46 => unkown,
        // 0x47 => Some(Key::Numlock),
        //0x48 => KeypadClear,
        // 0x49 => Some(Key::VolumeUp),
        // 0x4a => Some(Key::VolumeDown),
        // 0x4b => Some(Key::Divide),
        // 0x4c => Some(Key::NumpadEnter),
        //0x4d => unkown,
        0x4e => Some(Key::Minus),
        //0x4f => F18 key,
        //0x50 => F19 Key,
        // 0x51 => Some(Key::NumpadEquals),
        0x52 => Some(Key::Kp0),
        0x53 => Some(Key::Kp1),
        0x54 => Some(Key::Kp2),
        0x55 => Some(Key::Kp3),
        0x56 => Some(Key::Kp4),
        0x57 => Some(Key::Kp5),
        0x58 => Some(Key::Kp6),
        0x59 => Some(Key::Kp7),
        //0x5a => F20 Key,
        0x5b => Some(Key::Kp8),
        0x5c => Some(Key::Kp9),
        //0x5d => unkown,
        //0x5e => unkown,
        //0x5f => unkown,
        0x60 => Some(Key::F5),
        0x61 => Some(Key::F6),
        0x62 => Some(Key::F7),
        0x63 => Some(Key::F3),
        0x64 => Some(Key::F8),
        0x65 => Some(Key::F9),
        //0x66 => unkown,
        0x67 => Some(Key::F11),
        //0x68 => unkown,
        // 0x69 => Some(Key::F13),
        //0x6a => F16 Key,
        // 0x6b => Some(Key::F14),
        //0x6c => unkown,
        0x6d => Some(Key::F10),
        //0x6e => unkown,
        0x6f => Some(Key::F12),
        //0x70 => unkown,
        // 0x71 => Some(Key::F15),
        0x72 => Some(Key::Insert),
        0x73 => Some(Key::Home),
        0x74 => Some(Key::PageUp),
        0x75 => Some(Key::Delete),
        0x76 => Some(Key::F4),
        0x77 => Some(Key::End),
        0x78 => Some(Key::F2),
        0x79 => Some(Key::PageDown),
        0x7a => Some(Key::F1),
        0x7b => Some(Key::Left),
        0x7c => Some(Key::Right),
        0x7d => Some(Key::Down),
        0x7e => Some(Key::Up),
        //0x7f =>  unkown,
        _ => None,
    }
}

fn to_mods(event: id) -> KeyModifiers {
    let flags = unsafe { NSEvent::modifierFlags(event) };
    let mut result = KeyModifiers::empty();
    if flags.contains(appkit::NSShiftKeyMask) {
        result.insert(SHIFT);
    }
    if flags.contains(appkit::NSControlKeyMask) {
        result.insert(CONTROL);
    }
    if flags.contains(appkit::NSAlternateKeyMask) {
        result.insert(ALT);
    }
    if flags.contains(appkit::NSCommandKeyMask) {
        result.insert(SUPER);
    }
    result
}

fn is_printable(key_code: Key) -> bool {
    use self::Key::*;
    match key_code {
        Space | Apostrophe | Comma | Minus | Period | Slash | Num0 | Num1 | Num2 | Num3 |
        Num4 | Num5 | Num6 | Num7 | Num8 | Num9 | Semicolon | Equal | A | B | C | D | E | F |
        G | H | I | J | K | L | M | N | O | P | Q | R | S | T | U | V | W | X | Y | Z |
        LeftBracket | Backslash | RightBracket | GraveAccent | Tab | KpDecimal | KpDivide |
        KpMultiply | KpSubtract | KpAdd | KpEqual => true,
        _ => false,
    }
}

fn cursor_coordinates_in_view(view: &Object, nsevent: id) -> (i32, i32) {
    unsafe {
        let nswindow = nsevent.window();
        let window_point = nsevent.locationInWindow();
        let view_point: NSPoint = msg_send![view, convertPoint:window_point fromView:nil];
        let frame: NSRect = msg_send![view, frame];
        let hidpi_factor: CGFloat = msg_send![nswindow, backingScaleFactor];
        let hidpi_factor = hidpi_factor as f32;
        let x = (hidpi_factor * view_point.x as f32) as i32;
        let y = (hidpi_factor * (frame.size.height - view_point.y) as f32) as i32;
        (x, y)
    }
}
