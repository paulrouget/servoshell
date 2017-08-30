/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use glutin;
use servo::EventLoopWaker;
use std::cell::Cell;
use std::rc::Rc;

mod app;
mod logs;
mod utils;
mod view;
mod window;

pub use self::app::App;
pub use self::logs::Logger;
pub use self::view::View;
pub use self::window::Window;

use gleam::gl;
use view::{KeyModifiers, KeyState, ViewEvent, TouchPhase, ElementState, MouseButton, MouseScrollDelta};
use view::{SHIFT, CONTROL, ALT, SUPER};
use window::{WindowCommand, WindowEvent};

pub struct GlutinWindow {
    gl: Rc<gl::Gl>,
    glutin_window: glutin::GlWindow,
    event_loop_waker: Box<EventLoopWaker>,
    pending_key_event_char: Cell<Option<char>>,
    pressed_key_map: Vec<(glutin::ScanCode, char)>,
    view_events: Vec<ViewEvent>,
    window_events: Vec<WindowEvent>,
}

impl GlutinWindow {

    pub fn glutin_event_to_command(&self, event: &glutin::WindowEvent) -> Option<WindowCommand> {
        match *event {
            glutin::WindowEvent::KeyboardInput{ device_id: _device_id, input: glutin::KeyboardInput {
                state: glutin::ElementState::Pressed,
                scancode: _,
                virtual_keycode,
                modifiers
            }} => {
                match (virtual_keycode, utils::cmd_or_ctrl(modifiers), modifiers.ctrl, modifiers.shift) {
                    (Some(glutin::VirtualKeyCode::R), true, _, _) => Some(WindowCommand::Reload),
                    (Some(glutin::VirtualKeyCode::Left), true, _, _) => Some(WindowCommand::NavigateBack),
                    (Some(glutin::VirtualKeyCode::Right), true, _, _) => Some(WindowCommand::NavigateForward),
                    (Some(glutin::VirtualKeyCode::L), true, _, _) => Some(WindowCommand::OpenLocation),
                    (Some(glutin::VirtualKeyCode::Equals), true, _, _) => Some(WindowCommand::ZoomIn),
                    (Some(glutin::VirtualKeyCode::Minus), true, _, _) => Some(WindowCommand::ZoomOut),
                    (Some(glutin::VirtualKeyCode::Key0), true, _, _) => Some(WindowCommand::ZoomToActualSize),
                    (Some(glutin::VirtualKeyCode::T), true, _, _) => Some(WindowCommand::NewTab),
                    (Some(glutin::VirtualKeyCode::W), true, _, _) => Some(WindowCommand::CloseTab),
                    (Some(glutin::VirtualKeyCode::Tab), _, true, false) => Some(WindowCommand::NextTab),
                    (Some(glutin::VirtualKeyCode::Tab), _, true, true) => Some(WindowCommand::PrevTab),
                    (Some(glutin::VirtualKeyCode::Key1), true, _, _) => Some(WindowCommand::SelectTab(0)),
                    (Some(glutin::VirtualKeyCode::Key2), true, _, _) => Some(WindowCommand::SelectTab(1)),
                    (Some(glutin::VirtualKeyCode::Key3), true, _, _) => Some(WindowCommand::SelectTab(2)),
                    (Some(glutin::VirtualKeyCode::Key4), true, _, _) => Some(WindowCommand::SelectTab(3)),
                    (Some(glutin::VirtualKeyCode::Key5), true, _, _) => Some(WindowCommand::SelectTab(4)),
                    (Some(glutin::VirtualKeyCode::Key6), true, _, _) => Some(WindowCommand::SelectTab(5)),
                    (Some(glutin::VirtualKeyCode::Key7), true, _, _) => Some(WindowCommand::SelectTab(6)),
                    (Some(glutin::VirtualKeyCode::Key8), true, _, _) => Some(WindowCommand::SelectTab(7)),
                    (Some(glutin::VirtualKeyCode::Key9), true, _, _) => Some(WindowCommand::SelectTab(8)),
                    _ => None
                }
            }
            _ => None
        }
    }

    pub fn glutin_event_to_view_event(&mut self, event: &glutin::WindowEvent) -> Option<ViewEvent> {
        match *event {
            glutin::WindowEvent::Resized(..) => {
                Some(ViewEvent::GeometryDidChange)
            }
            glutin::WindowEvent::MouseMoved{device_id: _device_id, position: (x, y)} => {
                Some(ViewEvent::MouseMoved(x as i32, y as i32))
            }
            glutin::WindowEvent::MouseWheel{device_id: _device_id, delta, phase} => {
                let delta = match delta {
                    // FIXME: magic value
                    glutin::MouseScrollDelta::LineDelta(dx, dy) => MouseScrollDelta::LineDelta(dx, dy),
                    glutin::MouseScrollDelta::PixelDelta(dx, dy) => MouseScrollDelta::PixelDelta(dx, dy),
                };
                let phase = match phase {
                    glutin::TouchPhase::Started => TouchPhase::Started,
                    glutin::TouchPhase::Moved => TouchPhase::Moved,
                    glutin::TouchPhase::Ended => TouchPhase::Ended,
                    // FIXME:
                    glutin::TouchPhase::Cancelled => TouchPhase::Ended,
                };
                Some(ViewEvent::MouseWheel(delta, phase))
            }
            glutin::WindowEvent::MouseInput{device_id: _device_id, state, button: glutin::MouseButton::Left} => {
                let state = match state {
                    glutin::ElementState::Released => ElementState::Released,
                    glutin::ElementState::Pressed => ElementState::Pressed,
                };
                Some(ViewEvent::MouseInput(state, MouseButton::Left))
            }
            glutin::WindowEvent::ReceivedCharacter(ch) => {
                if !ch.is_control() {
                    self.pending_key_event_char.set(Some(ch));
                }
                None
            }
            glutin::WindowEvent::KeyboardInput{ device_id: _device_id, input: glutin::KeyboardInput {
                state, scancode, virtual_keycode: Some(virtual_keycode), modifiers}
            } => {

                // FIXME: that might not work on Windows. Check Servo's ports
                let ch = match state {
                    glutin::ElementState::Pressed => {
                        let ch = self.pending_key_event_char.get().and_then(|ch| {
                            if utils::is_printable(virtual_keycode) {
                                Some(ch)
                            } else {
                                None
                            }
                        });
                        self.pending_key_event_char.set(None);
                        if let Some(ch) = ch { self.pressed_key_map.push((scancode, ch)); }
                        ch
                    }
                    glutin::ElementState::Released => {
                        let idx = self.pressed_key_map.iter().position(|&(code, _)| code == scancode);
                        idx.map(|idx| self.pressed_key_map.swap_remove(idx).1)
                    }
                };

                if let Ok(key) = utils::glutin_key_to_script_key(virtual_keycode) {
                    let state = match state {
                        glutin::ElementState::Pressed => KeyState::Pressed,
                        glutin::ElementState::Released => KeyState::Released,
                    };
                    let mut servo_mods = KeyModifiers::empty();
                    if modifiers.shift { servo_mods.insert(SHIFT); }
                    if modifiers.ctrl { servo_mods.insert(CONTROL); }
                    if modifiers.alt { servo_mods.insert(ALT); }
                    if modifiers.logo { servo_mods.insert(SUPER); }
                    Some(ViewEvent::KeyEvent(ch, key, state, servo_mods))
                } else {
                    None
                }
            }

            _ => {
                None /* FIXME */
            }
        }
    }
}

