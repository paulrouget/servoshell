/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app::AppEvent;
use gleam::gl;
use glutin::{self, GlContext};
use servo::{ServoCursor, EventLoopWaker};
use state::{AppState, DebugOptions, WindowState};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use view::{DrawableGeometry, Key, KeyModifiers, KeyState, ViewEvent, TouchPhase, ElementState, MouseButton, MouseScrollDelta};
use view::{SHIFT, CONTROL, ALT, SUPER};
use window::{WindowCommand, WindowEvent};
use tinyfiledialogs;

pub struct App {
    event_loop: RefCell<glutin::EventsLoop>,
    event_loop_waker: Box<EventLoopWaker>,
    windows: Rc<RefCell<HashMap<glutin::WindowId, GlutinWindow>>>,
}

impl App {
    pub fn new() -> Result<App, &'static str> {
        let event_loop = glutin::EventsLoop::new();
        let event_loop_waker = box WinitEventLoopWaker {
            proxy: Arc::new(event_loop.create_proxy())
        };
        let windows = Rc::new(RefCell::new(HashMap::new()));
        Ok(App {
            windows,
            event_loop: RefCell::new(event_loop),
            event_loop_waker,
        })
    }

    pub fn get_init_state() -> AppState {
        AppState {
            current_window_index: None,
            windows: Vec::new(),
            dark_theme: false,
            cursor: ServoCursor::Default,
        }
    }

    pub fn get_resources_path() -> Option<PathBuf> {
        // Try current directory. Used for example with "cargo run"
        let p = env::current_dir().unwrap();
        if p.join("servo_resources/").exists() {
            return Some(p.join("servo_resources/"));
        }

        // Maybe we run from an app bundle
        let p = env::current_exe().unwrap();
        let p = p.parent().unwrap();
        let p = p.parent().unwrap().join("Resources");

        if p.join("servo_resources/").exists() {
            return Some(p.join("servo_resources/"));
        }

        None
    }

    pub fn render(&self, state: &AppState) {
        let cursor = servo_cursor_to_glutin_cursor(state.cursor);
        let windows = self.windows.borrow();
        for (_, window) in windows.iter() {
            window.glutin_window.set_cursor(cursor);
        };
    }

    pub fn get_events(&self) -> Vec<AppEvent> {
        vec![]
    }

    pub fn run<F>(&self, mut callback: F) where F: FnMut() {
        self.event_loop.borrow_mut().run_forever(|e| {
            let mut call_callback = false;
            match e {
                glutin::Event::WindowEvent {event, window_id} => {
                    let mut windows = self.windows.borrow_mut();
                    match windows.get_mut(&window_id) {
                        Some(window) => {
                            match (*window).glutin_event_to_command(&event) {
                                Some(command) => {
                                    window.window_events.push(WindowEvent::DoCommand(command));
                                    call_callback = true;
                                }
                                None => {
                                    match (*window).glutin_event_to_view_event(&event) {
                                        Some(event) => {
                                            window.view_events.push(event);
                                            call_callback = true;
                                        }
                                        None => {
                                            warn!("Got unknown glutin event: {:?}", event);
                                        }
                                    }
                                }
                            }
                        },
                        None => {
                            warn!("Unexpected event ({:?} for unknown Windows ({:?})", event, window_id);
                        }
                    }
                },
                glutin::Event::Awakened => {
                    let mut windows = self.windows.borrow_mut();
                    for (_, window) in windows.iter_mut() {
                        window.window_events.push(WindowEvent::EventLoopAwaken);
                    };
                    call_callback = true;
                }
                _ => { }
            }
            if call_callback {
                callback();
            }
            // FIXME: ControlFlow::Break
            glutin::ControlFlow::Continue
        });
        callback()
    }

    pub fn create_window(&self) -> Result<Window, &'static str> {
        let window = glutin::WindowBuilder::new()
            .with_dimensions(1024, 768);
        let context = glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)))
            .with_vsync(true);
        let glutin_window = glutin::GlWindow::new(window, context, &*self.event_loop.borrow_mut()).unwrap();

        let gl = unsafe {
            glutin_window.context().make_current().expect("Couldn't make window current");
            gl::GlFns::load_with(|s| glutin_window.context().get_proc_address(s) as *const _)
        };

        gl.clear_color(1.0, 1.0, 1.0, 1.0);
        gl.clear(gl::COLOR_BUFFER_BIT);
        gl.finish();

        glutin_window.show();

        let id = glutin_window.id();

        self.windows.borrow_mut().insert(id, GlutinWindow {
            gl,
            glutin_window,
            event_loop_waker: self.event_loop_waker.clone(),
            pending_key_event_char: Cell::new(None),
            pressed_key_map: vec![],
            view_events: vec![],
            window_events: vec![],
        });

        Ok(Window {
            id: id,
            windows: self.windows.clone(),
        })
    }
}

pub struct WinitEventLoopWaker {
    proxy: Arc<glutin::EventsLoopProxy>
}

impl EventLoopWaker for WinitEventLoopWaker {
    fn clone(&self) -> Box<EventLoopWaker + Send> {
        box WinitEventLoopWaker {
            proxy: self.proxy.clone()
        }
    }
    fn wake(&self) {
        self.proxy.wakeup().expect("wakeup eventloop failed");
    }
}

pub struct GlutinWindow {
    gl: Rc<gl::Gl>,
    glutin_window: glutin::GlWindow,
    event_loop_waker: Box<EventLoopWaker>,
    pending_key_event_char: Cell<Option<char>>,
    pressed_key_map: Vec<(glutin::ScanCode, char)>,
    view_events: Vec<ViewEvent>,
    window_events: Vec<WindowEvent>,
}


// Some shortcuts use Cmd on Mac and Control on other systems.
#[cfg(target_os = "macos")]
fn cmd_or_ctrl(m: glutin::ModifiersState) -> bool {m.logo}
#[cfg(not(target_os = "macos"))]
fn cmd_or_ctrl(m: glutin::ModifiersState) -> bool {m.ctrl}

impl GlutinWindow {

    pub fn glutin_event_to_command(&self, event: &glutin::WindowEvent) -> Option<WindowCommand> {
        match *event {
            glutin::WindowEvent::KeyboardInput{ device_id: _device_id, input: glutin::KeyboardInput {
                state: glutin::ElementState::Pressed,
                scancode: _,
                virtual_keycode,
                modifiers
            }} => {
                match (virtual_keycode, cmd_or_ctrl(modifiers), modifiers.ctrl, modifiers.shift) {
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
                        let ch = self.pending_key_event_char.get().and_then(|ch| filter_nonprintable(ch, virtual_keycode));
                        self.pending_key_event_char.set(None);
                        if let Some(ch) = ch { self.pressed_key_map.push((scancode, ch)); }
                        ch
                    }
                    glutin::ElementState::Released => {
                        let idx = self.pressed_key_map.iter().position(|&(code, _)| code == scancode);
                        idx.map(|idx| self.pressed_key_map.swap_remove(idx).1)
                    }
                };

                if let Ok(key) = glutin_key_to_script_key(virtual_keycode) {
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

pub struct Window {
    id: glutin::WindowId,
    windows: Rc<RefCell<HashMap<glutin::WindowId, GlutinWindow>>>,
}

pub struct View {
    id: glutin::WindowId,
    windows: Rc<RefCell<HashMap<glutin::WindowId, GlutinWindow>>>,
}

impl Window {

    pub fn render(&self, state: &mut WindowState) {
        // FIXME: mut WindowState
        let text = state.browsers.iter().enumerate().fold("|".to_owned(), |f, (idx, b)| {
            let title = b.title.as_ref().and_then(|t| {
                if t.is_empty() { None } else { Some(t) }
            }).map_or("No Title", |t| t.as_str());
            let selected = if Some(idx) == state.current_browser_index { '>' } else { ' ' };
            let loading = if b.is_loading { '*' } else { ' ' };
            format!("{} {} {:15.15} {}|", f, selected, title, loading)
        });

        let mut windows = self.windows.borrow_mut();
        windows.get_mut(&self.id).unwrap().glutin_window.set_title(&text);

        if state.urlbar_focused {
            let url = format!("{}", state.browsers[state.current_browser_index.unwrap()]
                              .url.as_ref().map_or("", |t| t.as_str()));
            match tinyfiledialogs::input_box("Search or type URL", "Search or type URL", &url) {
                Some(input) => {
                    let win = windows.get_mut(&self.id).unwrap();
                    win.window_events.push(WindowEvent::DoCommand(WindowCommand::Load(input)));
                }
                None => { },
            }
            state.urlbar_focused = false;
        }
    }

    pub fn get_init_state() -> WindowState {
        WindowState {
            current_browser_index: None,
            browsers: Vec::new(),
            sidebar_is_open: false,
            logs_visible: false,
            status: None,
            urlbar_focused: false,
            options_open: false,
            title: "ServoShell".to_owned(),
            debug_options: DebugOptions {
                show_fragment_borders: false,
                parallel_display_list_building: false,
                show_parallel_layout: false,
                convert_mouse_to_touch: false,
                show_tiles_borders: false,
                wr_profiler: false,
                wr_texture_cache_debug: false,
                wr_render_target_debug: false,
            },
        }
    }

    pub fn create_view(&self) -> Result<View, &'static str> {
        Ok(View {
            id: self.id.clone(),
            windows: self.windows.clone(),
        })
    }

    pub fn create_event_loop_waker(&self) -> Box<EventLoopWaker> {
        let mut windows = self.windows.borrow_mut();
        windows.get_mut(&self.id).unwrap().event_loop_waker.clone()
    }

    pub fn get_events(&self) -> Vec<WindowEvent> {
        let mut windows = self.windows.borrow_mut();
        let win = windows.get_mut(&self.id).unwrap();
        let events = win.window_events.drain(..).collect();
        events
    }

    pub fn append_logs(&self, _logs: &Vec<TermLog>) {
    }

}

impl View {

    pub fn get_geometry(&self) -> DrawableGeometry {
        let windows = self.windows.borrow();
        let win = windows.get(&self.id).unwrap();
        let size = win.glutin_window.get_inner_size().expect("Failed to get window inner size.");
        DrawableGeometry {
            view_size: size,
            margins: (0, 0, 0, 0),
            position: win.glutin_window.get_position().expect("Failed to get window position."),
            hidpi_factor: win.glutin_window.hidpi_factor(),
        }
    }

    pub fn update_drawable(&self) {
        let windows = self.windows.borrow();
        let win = windows.get(&self.id).unwrap();
        let (w, h) = win.glutin_window.get_inner_size().expect("Failed to get window inner size.");
        win.glutin_window.resize(w, h);
    }

    // FIXME: should be controlled by state
    pub fn enter_fullscreen(&self) {
    }

    // FIXME: should be controlled by state
    pub fn exit_fullscreen(&self) {
        self.windows.borrow().get(&self.id).unwrap().glutin_window.swap_buffers().unwrap();
    }

    pub fn set_live_resize_callback<F>(&self, _callback: &F) where F: FnMut() {
        // FIXME
    }

    pub fn gl(&self) -> Rc<gl::Gl> {
        self.windows.borrow().get(&self.id).unwrap().gl.clone()
    }

    pub fn get_events(&self) -> Vec<ViewEvent> {
        let mut windows = self.windows.borrow_mut();
        let win = windows.get_mut(&self.id).unwrap();
        let events = win.view_events.drain(..).collect();
        events
    }

    pub fn swap_buffers(&self) {
        self.windows.borrow().get(&self.id).unwrap().glutin_window.swap_buffers().unwrap();
    }
}

pub struct TermLog;
pub struct Logger;

impl Logger {
    pub fn init() -> Logger {
        Logger
    }

    pub fn get_logs(&self) -> Vec<TermLog> {
        vec![]
    }
}



fn glutin_key_to_script_key(key: glutin::VirtualKeyCode) -> Result<Key, ()> {
    match key {
        glutin::VirtualKeyCode::A => Ok(Key::A),
        glutin::VirtualKeyCode::B => Ok(Key::B),
        glutin::VirtualKeyCode::C => Ok(Key::C),
        glutin::VirtualKeyCode::D => Ok(Key::D),
        glutin::VirtualKeyCode::E => Ok(Key::E),
        glutin::VirtualKeyCode::F => Ok(Key::F),
        glutin::VirtualKeyCode::G => Ok(Key::G),
        glutin::VirtualKeyCode::H => Ok(Key::H),
        glutin::VirtualKeyCode::I => Ok(Key::I),
        glutin::VirtualKeyCode::J => Ok(Key::J),
        glutin::VirtualKeyCode::K => Ok(Key::K),
        glutin::VirtualKeyCode::L => Ok(Key::L),
        glutin::VirtualKeyCode::M => Ok(Key::M),
        glutin::VirtualKeyCode::N => Ok(Key::N),
        glutin::VirtualKeyCode::O => Ok(Key::O),
        glutin::VirtualKeyCode::P => Ok(Key::P),
        glutin::VirtualKeyCode::Q => Ok(Key::Q),
        glutin::VirtualKeyCode::R => Ok(Key::R),
        glutin::VirtualKeyCode::S => Ok(Key::S),
        glutin::VirtualKeyCode::T => Ok(Key::T),
        glutin::VirtualKeyCode::U => Ok(Key::U),
        glutin::VirtualKeyCode::V => Ok(Key::V),
        glutin::VirtualKeyCode::W => Ok(Key::W),
        glutin::VirtualKeyCode::X => Ok(Key::X),
        glutin::VirtualKeyCode::Y => Ok(Key::Y),
        glutin::VirtualKeyCode::Z => Ok(Key::Z),

        glutin::VirtualKeyCode::Numpad0 => Ok(Key::Kp0),
        glutin::VirtualKeyCode::Numpad1 => Ok(Key::Kp1),
        glutin::VirtualKeyCode::Numpad2 => Ok(Key::Kp2),
        glutin::VirtualKeyCode::Numpad3 => Ok(Key::Kp3),
        glutin::VirtualKeyCode::Numpad4 => Ok(Key::Kp4),
        glutin::VirtualKeyCode::Numpad5 => Ok(Key::Kp5),
        glutin::VirtualKeyCode::Numpad6 => Ok(Key::Kp6),
        glutin::VirtualKeyCode::Numpad7 => Ok(Key::Kp7),
        glutin::VirtualKeyCode::Numpad8 => Ok(Key::Kp8),
        glutin::VirtualKeyCode::Numpad9 => Ok(Key::Kp9),

        glutin::VirtualKeyCode::Key0 => Ok(Key::Num0),
        glutin::VirtualKeyCode::Key1 => Ok(Key::Num1),
        glutin::VirtualKeyCode::Key2 => Ok(Key::Num2),
        glutin::VirtualKeyCode::Key3 => Ok(Key::Num3),
        glutin::VirtualKeyCode::Key4 => Ok(Key::Num4),
        glutin::VirtualKeyCode::Key5 => Ok(Key::Num5),
        glutin::VirtualKeyCode::Key6 => Ok(Key::Num6),
        glutin::VirtualKeyCode::Key7 => Ok(Key::Num7),
        glutin::VirtualKeyCode::Key8 => Ok(Key::Num8),
        glutin::VirtualKeyCode::Key9 => Ok(Key::Num9),

        glutin::VirtualKeyCode::Return => Ok(Key::Enter),
        glutin::VirtualKeyCode::Space => Ok(Key::Space),
        glutin::VirtualKeyCode::Escape => Ok(Key::Escape),
        glutin::VirtualKeyCode::Equals => Ok(Key::Equal),
        glutin::VirtualKeyCode::Minus => Ok(Key::Minus),
        glutin::VirtualKeyCode::Back => Ok(Key::Backspace),
        glutin::VirtualKeyCode::PageDown => Ok(Key::PageDown),
        glutin::VirtualKeyCode::PageUp => Ok(Key::PageUp),

        glutin::VirtualKeyCode::Insert => Ok(Key::Insert),
        glutin::VirtualKeyCode::Home => Ok(Key::Home),
        glutin::VirtualKeyCode::Delete => Ok(Key::Delete),
        glutin::VirtualKeyCode::End => Ok(Key::End),

        glutin::VirtualKeyCode::Left => Ok(Key::Left),
        glutin::VirtualKeyCode::Up => Ok(Key::Up),
        glutin::VirtualKeyCode::Right => Ok(Key::Right),
        glutin::VirtualKeyCode::Down => Ok(Key::Down),

        glutin::VirtualKeyCode::LShift => Ok(Key::LeftShift),
        glutin::VirtualKeyCode::LControl => Ok(Key::LeftControl),
        glutin::VirtualKeyCode::LAlt => Ok(Key::LeftAlt),
        glutin::VirtualKeyCode::LWin => Ok(Key::LeftSuper),
        glutin::VirtualKeyCode::RShift => Ok(Key::RightShift),
        glutin::VirtualKeyCode::RControl => Ok(Key::RightControl),
        glutin::VirtualKeyCode::RAlt => Ok(Key::RightAlt),
        glutin::VirtualKeyCode::RWin => Ok(Key::RightSuper),

        glutin::VirtualKeyCode::Apostrophe => Ok(Key::Apostrophe),
        glutin::VirtualKeyCode::Backslash => Ok(Key::Backslash),
        glutin::VirtualKeyCode::Comma => Ok(Key::Comma),
        glutin::VirtualKeyCode::Grave => Ok(Key::GraveAccent),
        glutin::VirtualKeyCode::LBracket => Ok(Key::LeftBracket),
        glutin::VirtualKeyCode::Period => Ok(Key::Period),
        glutin::VirtualKeyCode::RBracket => Ok(Key::RightBracket),
        glutin::VirtualKeyCode::Semicolon => Ok(Key::Semicolon),
        glutin::VirtualKeyCode::Slash => Ok(Key::Slash),
        glutin::VirtualKeyCode::Tab => Ok(Key::Tab),
        glutin::VirtualKeyCode::Subtract => Ok(Key::Minus),

        glutin::VirtualKeyCode::F1 => Ok(Key::F1),
        glutin::VirtualKeyCode::F2 => Ok(Key::F2),
        glutin::VirtualKeyCode::F3 => Ok(Key::F3),
        glutin::VirtualKeyCode::F4 => Ok(Key::F4),
        glutin::VirtualKeyCode::F5 => Ok(Key::F5),
        glutin::VirtualKeyCode::F6 => Ok(Key::F6),
        glutin::VirtualKeyCode::F7 => Ok(Key::F7),
        glutin::VirtualKeyCode::F8 => Ok(Key::F8),
        glutin::VirtualKeyCode::F9 => Ok(Key::F9),
        glutin::VirtualKeyCode::F10 => Ok(Key::F10),
        glutin::VirtualKeyCode::F11 => Ok(Key::F11),
        glutin::VirtualKeyCode::F12 => Ok(Key::F12),

        glutin::VirtualKeyCode::NavigateBackward => Ok(Key::NavigateBackward),
        glutin::VirtualKeyCode::NavigateForward => Ok(Key::NavigateForward),
        _ => Err(()),
    }
}

fn is_printable(key_code: glutin::VirtualKeyCode) -> bool {
    use glutin::VirtualKeyCode::*;
    match key_code {
        Escape |
            F1 |
            F2 |
            F3 |
            F4 |
            F5 |
            F6 |
            F7 |
            F8 |
            F9 |
            F10 |
            F11 |
            F12 |
            F13 |
            F14 |
            F15 |
            Snapshot |
            Scroll |
            Pause |
            Insert |
            Home |
            Delete |
            End |
            PageDown |
            PageUp |
            Left |
            Up |
            Right |
            Down |
            Back |
            LAlt |
            LControl |
            LMenu |
            LShift |
            LWin |
            Mail |
            MediaSelect |
            MediaStop |
            Mute |
            MyComputer |
            NavigateForward |
            NavigateBackward |
            NextTrack |
            NoConvert |
            PlayPause |
            Power |
            PrevTrack |
            RAlt |
            RControl |
            RMenu |
            RShift |
            RWin |
            Sleep |
            Stop |
            VolumeDown |
            VolumeUp |
            Wake |
            WebBack |
            WebFavorites |
            WebForward |
            WebHome |
            WebRefresh |
            WebSearch |
            WebStop => false,
        _ => true,
    }
}

fn filter_nonprintable(ch: char, key_code: glutin::VirtualKeyCode) -> Option<char> {
    if is_printable(key_code) {
        Some(ch)
    } else {
        None
    }
}

fn servo_cursor_to_glutin_cursor(servo_cursor: ServoCursor) -> glutin::MouseCursor {
    match servo_cursor {
        ServoCursor::None => glutin::MouseCursor::NoneCursor,
        ServoCursor::Default => glutin::MouseCursor::Default,
        ServoCursor::Pointer => glutin::MouseCursor::Hand,
        ServoCursor::ContextMenu => glutin::MouseCursor::ContextMenu,
        ServoCursor::Help => glutin::MouseCursor::Help,
        ServoCursor::Progress => glutin::MouseCursor::Progress,
        ServoCursor::Wait => glutin::MouseCursor::Wait,
        ServoCursor::Cell => glutin::MouseCursor::Cell,
        ServoCursor::Crosshair => glutin::MouseCursor::Crosshair,
        ServoCursor::Text => glutin::MouseCursor::Text,
        ServoCursor::VerticalText => glutin::MouseCursor::VerticalText,
        ServoCursor::Alias => glutin::MouseCursor::Alias,
        ServoCursor::Copy => glutin::MouseCursor::Copy,
        ServoCursor::Move => glutin::MouseCursor::Move,
        ServoCursor::NoDrop => glutin::MouseCursor::NoDrop,
        ServoCursor::NotAllowed => glutin::MouseCursor::NotAllowed,
        ServoCursor::Grab => glutin::MouseCursor::Grab,
        ServoCursor::Grabbing => glutin::MouseCursor::Grabbing,
        ServoCursor::EResize => glutin::MouseCursor::EResize,
        ServoCursor::NResize => glutin::MouseCursor::NResize,
        ServoCursor::NeResize => glutin::MouseCursor::NeResize,
        ServoCursor::NwResize => glutin::MouseCursor::NwResize,
        ServoCursor::SResize => glutin::MouseCursor::SResize,
        ServoCursor::SeResize => glutin::MouseCursor::SeResize,
        ServoCursor::SwResize => glutin::MouseCursor::SwResize,
        ServoCursor::WResize => glutin::MouseCursor::WResize,
        ServoCursor::EwResize => glutin::MouseCursor::EwResize,
        ServoCursor::NsResize => glutin::MouseCursor::NsResize,
        ServoCursor::NeswResize => glutin::MouseCursor::NeswResize,
        ServoCursor::NwseResize => glutin::MouseCursor::NwseResize,
        ServoCursor::ColResize => glutin::MouseCursor::ColResize,
        ServoCursor::RowResize => glutin::MouseCursor::RowResize,
        ServoCursor::AllScroll => glutin::MouseCursor::AllScroll,
        ServoCursor::ZoomIn => glutin::MouseCursor::ZoomIn,
        ServoCursor::ZoomOut => glutin::MouseCursor::ZoomOut,
    }
}
