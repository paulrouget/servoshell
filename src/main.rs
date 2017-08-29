/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(link_args)]


#[macro_use]
extern crate log;

#[cfg(feature = "with-cocoa")]
extern crate libc;
#[cfg(feature = "with-cocoa")]
extern crate cocoa;
#[cfg(feature = "with-cocoa")]
extern crate objc_foundation;
#[cfg(feature = "with-cocoa")]
#[macro_use]
extern crate objc;

#[cfg(feature = "with-glutin")]
extern crate glutin;
#[cfg(feature = "with-glutin")]
extern crate winit;
#[cfg(feature = "with-glutin")]
extern crate tinyfiledialogs;

extern crate gleam;
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
use std::cell::RefCell;
use std::rc::Rc;
use std::env::args;
use servo::{Servo, ServoUrl, WebRenderDebugOption};

fn main() {

    let logs = platform::Logger::init();

    info!("starting");

    let app = App::new().unwrap();
    let window = app.create_window().unwrap();

    let view = Rc::new(window.create_view().unwrap());

    Servo::configure(App::get_resources_path().unwrap());

    let servo = {
        let geometry = view.get_geometry();
        let waker = window.create_event_loop_waker();
        Servo::new(geometry, view.clone(), waker)
    };

    let home_url = App::get_resources_path().unwrap().parent().unwrap().join("shell_resources").join("home.html");
    let home_url = ServoUrl::from_file_path(&home_url).unwrap().into_string();

    // Skip first argument (executable), and find the first
    // argument that doesn't start with `-`
    let url = args().skip(1).find(|arg| {
        !arg.starts_with("-")
    }).unwrap_or(home_url);

    let browser = servo.create_browser(&url);
    servo.select_browser(browser.id);

    let mut state = App::get_init_state();
    state.current_window_index = Some(0);
    state.windows.push(Window::get_init_state());
    state.windows[0].current_browser_index = Some(0);
    state.windows[0].browsers.push(browser);
    app.render(&state);
    // FIXME: no mut!
    window.render(&mut state.windows[0]);

    info!("Servo version: {}", servo.version());

    // Use a RefCell, otherwise the handle_events callback needs to be a FnMut, making things a bit hard
    // FIXME: do not use a RefCell
    let state = RefCell::new(state);

    let handle_events = || {

        let mut state = state.borrow_mut();

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
                                state.dark_theme = !state.dark_theme;
                                ui_invalidated = true;
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
                        let idx = state.windows[0].current_browser_index.unwrap();
                        let bid = state.windows[0].browsers[idx].id;
                        match cmd {
                            WindowCommand::Stop => {
                                // FIXME
                            }
                            WindowCommand::Reload => {
                                servo.reload(bid);
                            }
                            WindowCommand::NavigateBack => {
                                servo.go_back(bid);
                            }
                            WindowCommand::NavigateForward => {
                                servo.go_forward(bid);
                            }
                            WindowCommand::OpenLocation => {
                                state.windows[0].urlbar_focused = true;
                                ui_invalidated = true;
                            }
                            WindowCommand::OpenInDefaultBrowser => {
                                if let Some(ref url) = state.windows[0].browsers[idx].url {
                                    open::that(url.clone()).ok();
                                }
                            }
                            WindowCommand::ZoomIn => {
                                ui_invalidated = true;
                                state.windows[0].browsers[idx].zoom *= 1.1;
                                servo.zoom(state.windows[0].browsers[idx].zoom);
                            }
                            WindowCommand::ZoomOut => {
                                ui_invalidated = true;
                                state.windows[0].browsers[idx].zoom /= 1.1;
                                servo.zoom(state.windows[0].browsers[idx].zoom);
                            }
                            WindowCommand::ZoomToActualSize => {
                                ui_invalidated = true;
                                state.windows[0].browsers[idx].zoom = 1.0;
                                servo.reset_zoom();
                            }

                            WindowCommand::ToggleSidebar => {
                                state.windows[0].sidebar_is_open = !state.windows[0].sidebar_is_open;
                                ui_invalidated = true;
                            }

                            WindowCommand::ShowOptions => {
                                state.windows[0].options_open = !state.windows[0].options_open;
                                ui_invalidated = true;
                            }

                            WindowCommand::Load(request) => {
                                state.windows[0].browsers[idx].user_input = Some(request.clone());
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
                                        servo.load_url(bid, url)
                                    },
                                    Err(err) => warn!("Can't parse url: {}", err),
                                }
                            }
                            WindowCommand::ToggleOptionShowLogs => {
                                state.windows[0].logs_visible = !state.windows[0].logs_visible;
                                ui_invalidated = true;
                            },
                            WindowCommand::NewTab => {
                                let browser = servo.create_browser("about:blank");
                                servo.select_browser(browser.id);
                                servo.update_geometry(view.get_geometry());
                                state.windows[0].current_browser_index = Some(idx + 1);
                                state.windows[0].browsers.push(browser);
                                ui_invalidated = true;
                            },
                            WindowCommand::CloseTab => {
                                if state.windows[0].browsers.len() > 1 {
                                    let id = state.windows[0].browsers[idx].id;
                                    let new_id = if idx == state.windows[0].browsers.len() - 1 {
                                        state.windows[0].current_browser_index = Some(idx - 1);
                                        state.windows[0].browsers[idx - 1].id
                                    } else {
                                        state.windows[0].browsers[idx + 1].id
                                    };
                                    servo.select_browser(new_id);
                                    servo.close_browser(id);
                                    state.windows[0].browsers.remove(idx);
                                    ui_invalidated = true;
                                }
                            },
                            WindowCommand::PrevTab => {
                                let new_idx = if idx == 0 {
                                    state.windows[0].browsers.len() - 1
                                } else {
                                    idx - 1
                                };
                                state.windows[0].current_browser_index = Some(new_idx);
                                let id = state.windows[0].browsers[new_idx].id;
                                servo.select_browser(id);
                                ui_invalidated = true;
                            },
                            WindowCommand::NextTab => {
                                let new_idx = if idx == state.windows[0].browsers.len() - 1 {
                                    0
                                } else {
                                    idx + 1
                                };
                                state.windows[0].current_browser_index = Some(new_idx);
                                let id = state.windows[0].browsers[new_idx].id;
                                servo.select_browser(id);
                                ui_invalidated = true;
                            },
                            WindowCommand::SelectTab(idx) => {
                                if state.windows[0].current_browser_index != Some(idx) {
                                    state.windows[0].current_browser_index = Some(idx);
                                    let id = state.windows[0].browsers[idx].id;
                                    servo.select_browser(id);
                                    ui_invalidated = true;
                                }
                            },
                            WindowCommand::ToggleOptionFragmentBorders => { },
                            WindowCommand::ToggleOptionParallelDisplayListBuidling => { },
                            WindowCommand::ToggleOptionShowParallelLayout => { },
                            WindowCommand::ToggleOptionConvertMouseToTouch => { },
                            WindowCommand::ToggleOptionTileBorders => { },

                            WindowCommand::ToggleOptionWRProfiler => {
                                state.windows[0].debug_options.wr_profiler = !state.windows[0].debug_options.wr_profiler;
                                servo.toggle_webrender_debug_option(WebRenderDebugOption::Profiler);
                            },

                            WindowCommand::ToggleOptionWRTextureCacheDebug => {
                                state.windows[0].debug_options.wr_texture_cache_debug = !state.windows[0].debug_options.wr_texture_cache_debug;
                                servo.toggle_webrender_debug_option(WebRenderDebugOption::TextureCacheDebug);
                            },

                            WindowCommand::ToggleOptionWRTargetDebug => {
                                state.windows[0].debug_options.wr_render_target_debug = !state.windows[0].debug_options.wr_render_target_debug;
                                servo.toggle_webrender_debug_option(WebRenderDebugOption::RenderTargetDebug);
                            },
                        }
                    }
                }
            }

            for event in view_events {
                let idx = state.windows[0].current_browser_index.unwrap();
                let state = &mut state.windows[0].browsers[idx];
                match event {
                    ViewEvent::GeometryDidChange => {
                        servo.update_geometry(view.get_geometry());
                        view.update_drawable();
                    }
                    ViewEvent::MouseWheel(delta, phase) => {
                        let (mut x, mut y) = match delta {
                            view::MouseScrollDelta::PixelDelta(x, y) => (x, y),
                            _ => (0.0, 0.0),
                        };
                        if y.abs() >= x.abs() { x = 0.0; } else { y = 0.0; }
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
                    ViewEvent::KeyEvent(c, key, keystate, modifiers) => {
                        servo.send_key(state.id, c, key, keystate, modifiers);
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
                        match state.windows[0].browsers.iter_mut().find(|b| b.id == id) {
                            Some(browser) => {
                                browser.title = title;
                                ui_invalidated = true;
                            }
                            None => { /*FIXME*/ }
                        }
                    }
                    ServoEvent::StatusChanged(status) => {
                        state.windows[0].status = status;
                        ui_invalidated = true;
                    }
                    ServoEvent::LoadStart(id) => {
                        match state.windows[0].browsers.iter_mut().find(|b| b.id == id) {
                            Some(browser) => {
                                browser.is_loading = true;
                                ui_invalidated = true;
                            }
                            None => { /*FIXME*/ }
                        }
                    }
                    ServoEvent::LoadEnd(id) => {
                        match state.windows[0].browsers.iter_mut().find(|b| b.id == id) {
                            Some(browser) => {
                                browser.is_loading = false;
                                ui_invalidated = true;
                            }
                            None => { /*FIXME*/ }
                        }
                    }
                    ServoEvent::HeadParsed(..) => {
                        // FIXME
                    }
                    ServoEvent::HistoryChanged(id, entries, current) => {
                        match state.windows[0].browsers.iter_mut().find(|b| b.id == id) {
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
                        state.cursor = cursor;
                        ui_invalidated = true;
                    }
                    ServoEvent::FaviconChanged(..) => {
                        // FIXME
                    }
                    ServoEvent::Key(..) => {
                        // FIXME
                    }
                    ServoEvent::OpenInDefaultBrowser(url) => {
                        open::that(url).ok();
                    }
                }
            }

            if ui_invalidated {
                app.render(&state);
                // FIXME: no mut!
                window.render(&mut state.windows[0]);
            }

            servo.sync(force_sync);
        }

        // Here, only stuff that we know for sure won't trigger any
        // new events

        // FIXME: logs will grow until pulled
        if state.windows[0].logs_visible {
            window.append_logs(&logs.get_logs());
        }
    };

    view.set_live_resize_callback(&handle_events);

    app.run(handle_events);

}
