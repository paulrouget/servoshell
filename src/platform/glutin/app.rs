/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app::AppEvent;
use gleam::gl;
use glutin::{self, GlContext};
use servo::{ServoCursor, EventLoopWaker};
use state::AppState;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use super::GlutinWindow;
use super::utils;
use window::{Window, WindowEvent};

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
        let cursor = utils::servo_cursor_to_glutin_cursor(state.cursor);
        let windows = self.windows.borrow();
        for (_, window) in windows.iter() {
            window.glutin_window.set_cursor(cursor);
        };
    }

    pub fn get_events(&self) -> Vec<AppEvent> {
        vec![]
    }

    fn should_exit(&self, event: &glutin::WindowEvent) -> bool {
        // Exit if window is closed or if Cmd/Ctrl Q
        match *event {
            glutin::WindowEvent::Closed => {
                return true
            },
            _ => { }
        }

        if let glutin::WindowEvent::KeyboardInput {
            device_id: _,
            input: glutin::KeyboardInput {
                state: glutin::ElementState::Pressed,
                scancode: _,
                virtual_keycode: Some(glutin::VirtualKeyCode::Q),
                modifiers,
            }
        } = *event {
            if utils::cmd_or_ctrl(modifiers) {
                return true
            }
        }
        false
    }

    pub fn run<F>(&self, mut callback: F) where F: FnMut() {
        self.event_loop.borrow_mut().run_forever(|e| {
            let mut call_callback = false;
            match e {
                glutin::Event::WindowEvent {event, window_id} => {
                    if self.should_exit(&event) {
                        return glutin::ControlFlow::Break;
                    }
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

        Ok(Window::new(id, self.windows.clone()))
    }
}


