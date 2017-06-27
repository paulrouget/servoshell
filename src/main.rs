/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]

#[macro_use]
extern crate objc;

#[macro_use]
extern crate log;

extern crate libc;
extern crate cocoa;
extern crate gleam;
extern crate objc_foundation;
extern crate open;

mod app;
mod window;
mod view;
mod servo;
mod platform;
mod state;

use app::{App, AppEvent, AppCommand};
use window::{Window, WindowEvent, WindowCommand};
use view::ViewEvent;
use servo::ServoEvent;
use std::rc::Rc;
use std::env::args;
use servo::{Servo, ServoUrl};

use platform::get_state;

fn main() {

    let logs = platform::Logger::init();

    info!("starting");

    platform::init();

    let app = App::load().unwrap();
    let window = app.create_window().unwrap();

    get_state().current_window_index = Some(0);
    get_state().window_states.push(Window::get_init_state());

    let view = Rc::new(window.create_view().unwrap());

    Servo::configure().unwrap();
    let servo = {
        let geometry = view.get_geometry();
        let waker = window.create_event_loop_waker();
        Servo::new(geometry, view.clone(), waker)
    };

    // Skip first argument (executable), and find the first
    // argument that doesn't start with `-`
    let url = args().skip(1).find(|arg| {
        !arg.starts_with("-")
    }).unwrap_or("https://blog.servo.org/".to_owned());

    let browser = servo.create_browser(&url);
    servo.select_browser(browser.id);

    get_state().window_states[0].current_browser_index = Some(0);
    get_state().window_states[0].browser_states.push(browser);

    info!("Servo version: {}", servo.version());

    let handle_events = || {

        // Loop until no events are available anymore.
        loop {

            let app_events = app.get_events();
            let win_events = window.get_events();
            let view_events = view.get_events();
            let servo_events = servo.get_events();

            if app_events.is_empty() &&
               win_events.is_empty() &&
               view_events.is_empty() &&
               servo_events.is_empty() {
                   break
            }

            // FIXME: it's really annoying we need this
            let mut force_sync = false;
            let mut ui_invalidated = false;

            for event in app_events {
                match event {
                    AppEvent::DidFinishLaunching => {
                        // FIXME: does this work?
                    }
                    AppEvent::WillTerminate => {
                        // FIXME: does this work?
                    }
                    AppEvent::DidChangeScreenParameters => {
                        // FIXME: does this work?
                        servo.update_geometry(view.get_geometry());
                        view.update_drawable();
                    }
                    AppEvent::DoCommand(cmd) => {
                        match cmd {
                            AppCommand::ClearHistory => {
                                // FIXME
                            }
                            AppCommand::ToggleOptionDarkTheme => {
                                ui_invalidated = true;
                                get_state().dark_theme = !get_state().dark_theme;
                                window.update_theme();
                            }
                        }
                    }
                }
            }

            for event in win_events {
                match event {
                    WindowEvent::EventLoopAwaken => {
                        force_sync = true;
                    }
                    WindowEvent::GeometryDidChange => {
                        servo.update_geometry(view.get_geometry());
                        view.update_drawable();
                    }
                    WindowEvent::DidEnterFullScreen => {
                        // FIXME
                    }
                    WindowEvent::DidExitFullScreen => {
                        // FIXME
                    }
                    WindowEvent::WillClose => {
                        // FIXME
                    }
                    WindowEvent::DoCommand(cmd) => {
                        let idx = get_state().window_states[0].current_browser_index.unwrap();
                        let ref mut state = get_state().window_states[0].browser_states[idx];
                        match cmd {
                            WindowCommand::Stop => {
                                // FIXME
                            }
                            WindowCommand::Reload => {
                                servo.reload(state.id);
                            }
                            WindowCommand::NavigateBack => {
                                servo.go_back(state.id);
                            }
                            WindowCommand::NavigateForward => {
                                servo.go_forward(state.id);
                            }
                            WindowCommand::OpenLocation => {
                                window.focus_urlbar();
                            }
                            WindowCommand::OpenInDefaultBrowser => {
                                if let Some(ref url) = state.url {
                                    open::that(url.clone()).ok();
                                }
                            }
                            WindowCommand::ToggleSidebar => {
                                window.toggle_sidebar();
                            }
                            WindowCommand::ZoomIn => {
                                ui_invalidated = true;
                                state.zoom *= 1.1;
                                servo.zoom(state.zoom);
                            }
                            WindowCommand::ZoomOut => {
                                ui_invalidated = true;
                                state.zoom /= 1.1;
                                servo.zoom(state.zoom);
                            }
                            WindowCommand::ZoomToActualSize => {
                                ui_invalidated = true;
                                state.zoom = 1.0;
                                servo.reset_zoom();
                            }
                            WindowCommand::ShowOptions => {
                                window.show_options();
                            }
                            WindowCommand::Load(request) => {
                                state.user_input = Some(request.clone());
                                let url = ServoUrl::parse(&request).or_else(|error| {
                                    // FIXME: weak
                                    if request.ends_with(".com") || request.ends_with(".org") || request.ends_with(".net") {
                                        ServoUrl::parse(&format!("http://{}", request))
                                    } else {
                                        Err(error)
                                    }
                                }).or_else(|_| {
                                    ServoUrl::parse(&format!("https://duckduckgo.com/html/?q={}", request))
                                });
                                match url {
                                    Ok(url) => {
                                        servo.load_url(state.id, url)
                                    },
                                    Err(err) => warn!("Can't parse url: {}", err),
                                }
                            }
                            WindowCommand::ToggleOptionShowLogs => {
                                get_state().window_states[0].logs_visible = !get_state().window_states[0].logs_visible;
                                ui_invalidated = true;
                            },
                            WindowCommand::ToggleOptionLockDomain => {
                                state.domain_locked = !state.domain_locked;
                                if state.domain_locked {
                                    let url = ServoUrl::parse(state.url.as_ref().unwrap()).unwrap();
                                    let domain = url.domain().unwrap();
                                    servo.limit_to_domain(Some(domain.to_owned()));
                                } else {
                                    servo.limit_to_domain(None);
                                }
                            },
                            WindowCommand::NewTab => {
                                let browser = servo.create_browser("about:blank");
                                servo.select_browser(browser.id);
                                servo.update_geometry(view.get_geometry());
                                get_state().window_states[0].current_browser_index = Some(idx + 1);
                                get_state().window_states[0].browser_states.push(browser);
                                ui_invalidated = true;
                            },
                            WindowCommand::PrevTab => {
                                let new_idx = if idx == 0 {
                                    get_state().window_states[0].browser_states.len() - 1
                                } else {
                                    idx - 1
                                };
                                get_state().window_states[0].current_browser_index = Some(new_idx);
                                let id = get_state().window_states[0].browser_states[new_idx].id;
                                servo.select_browser(id);
                                ui_invalidated = true;
                            },
                            WindowCommand::NextTab => {
                                let new_idx = if idx == get_state().window_states[0].browser_states.len() - 1 {
                                    0
                                } else {
                                    idx + 1
                                };
                                get_state().window_states[0].current_browser_index = Some(new_idx);
                                let id = get_state().window_states[0].browser_states[new_idx].id;
                                servo.select_browser(id);
                                ui_invalidated = true;
                            },
                            WindowCommand::ToggleOptionFragmentBorders => { },
                            WindowCommand::ToggleOptionParallelDisplayListBuidling => { },
                            WindowCommand::ToggleOptionShowParallelLayout => { },
                            WindowCommand::ToggleOptionConvertMouseToTouch => { },
                            WindowCommand::ToggleOptionWebRenderStats => {
                                let ref mut state = get_state().window_states[0].browser_states[0];
                                state.show_webrender_stats = !state.show_webrender_stats;
                                servo.set_webrender_profiler_enabled(state.show_webrender_stats);
                            },
                            WindowCommand::ToggleOptionTileBorders => { },
                        }
                    }
                }
            }

            for event in view_events {
                let idx = get_state().window_states[0].current_browser_index.unwrap();
                let ref mut state = get_state().window_states[0].browser_states[idx];
                match event {
                    ViewEvent::GeometryDidChange => {
                        servo.update_geometry(view.get_geometry());
                        view.update_drawable();
                    }
                    ViewEvent::MouseWheel(delta, phase) => {
                        let (x, y) = match delta {
                            view::MouseScrollDelta::PixelDelta(x, y) => {
                                (x, y)
                            },
                            _ => (0.0, 0.0),
                        };
                        servo.perform_scroll(0, 0, x, y, phase);
                    }
                    ViewEvent::MouseMoved(x, y) => {
                        state.last_mouse_point = (x, y);
                        servo.perform_mouse_move(x, y);
                    }
                    ViewEvent::MouseInput(element_state, button) => {
                        let (x, y) = state.last_mouse_point;
                        let (org_x, org_y) = state.last_mouse_down_point;
                        let last_mouse_down_button = state.last_mouse_down_button;
                        servo.perform_click(x, y, org_x, org_y, element_state, button, last_mouse_down_button);
                        state.last_mouse_down_point = (x, y);
                        if element_state == view::ElementState::Pressed {
                            state.last_mouse_down_button = Some(button);
                        }
                    }
                }
            }

            for event in servo_events {
                match event {
                    ServoEvent::SetWindowInnerSize(..) => {
                        // ignore
                    }
                    ServoEvent::SetWindowPosition(..) => {
                        // ignore
                    }
                    ServoEvent::SetFullScreenState(fullscreen) => {
                        if fullscreen {
                            view.enter_fullscreen();
                        } else {
                            view.exit_fullscreen();
                        }
                    }
                    ServoEvent::TitleChanged(id, title) => {
                        match get_state().window_states[0].browser_states.iter_mut().find(|b| b.id == id) {
                            Some(mut browser) => {
                                browser.title = title;
                                ui_invalidated = true;
                            }
                            None => { /*FIXME*/ }
                        }
                    }
                    ServoEvent::UnhandledURL(url) => {
                        open::that(url.as_str()).ok();

                    }
                    ServoEvent::StatusChanged(status) => {
                        window.set_status(status);
                    }
                    ServoEvent::LoadStart(id) => {
                        match get_state().window_states[0].browser_states.iter_mut().find(|b| b.id == id) {
                            Some(browser) => {
                                browser.is_loading = true;
                                ui_invalidated = true;
                            }
                            None => { /*FIXME*/ }
                        }
                    }
                    ServoEvent::LoadEnd(id) => {
                        match get_state().window_states[0].browser_states.iter_mut().find(|b| b.id == id) {
                            Some(browser) => {
                                browser.is_loading = false;
                                ui_invalidated = true;
                            }
                            None => { /*FIXME*/ }
                        }
                    }
                    ServoEvent::LoadError(..) => {
                        // FIXME
                    }
                    ServoEvent::HeadParsed(..) => {
                        // FIXME
                    }
                    ServoEvent::HistoryChanged(id, entries, current) => {
                        match get_state().window_states[0].browser_states.iter_mut().find(|b| b.id == id) {
                            Some(browser) => {
                                let url = entries[current].url.to_string();
                                browser.url = Some(url);
                                browser.can_go_back = current > 0;
                                browser.can_go_forward = current < entries.len() - 1;
                                ui_invalidated = true;
                            }
                            None => { /*FIXME*/ }
                        }
                    }
                    ServoEvent::CursorChanged(cursor) => {
                        window.set_cursor(cursor);
                    }
                    ServoEvent::FaviconChanged(id, url) => {
                        // FIXME
                    }
                    ServoEvent::Key(..) => {
                        // FIXME
                    }
                }
            }

            if ui_invalidated {
                app.state_changed();
                window.state_changed();
            }

            servo.sync(force_sync);
        }

        // Here, only stuff that we know for sure won't trigger any
        // new events

        // FIXME: logs will grow until pulled
        if get_state().window_states[0].logs_visible {
            window.append_logs(&logs.get_logs());
        }
    };

    view.set_live_resize_callback(&handle_events);

    app.run(handle_events);

}
