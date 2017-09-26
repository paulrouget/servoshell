/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(link_args)]
#![feature(slice_patterns)]

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate treediff;

#[macro_use]
extern crate log;

#[cfg(all(not(feature = "force-glutin"), target_os = "macos"))]
extern crate libc;
#[cfg(all(not(feature = "force-glutin"), target_os = "macos"))]
extern crate cocoa;
#[cfg(all(not(feature = "force-glutin"), target_os = "macos"))]
#[macro_use]
extern crate objc;

#[cfg(any(feature = "force-glutin", not(target_os = "macos")))]
extern crate glutin;
#[cfg(any(feature = "force-glutin", not(target_os = "macos")))]
extern crate tinyfiledialogs;


#[cfg(target_os = "windows")]
extern crate winapi;
#[cfg(target_os = "windows")]
extern crate user32;
#[cfg(target_os = "windows")]
extern crate gdi32;

extern crate open;

mod traits;
mod platform;
mod servo;
mod state;
mod logs;

use platform::App;
use servo::{Servo, ServoEvent, ServoUrl, WebRenderDebugOption};
use std::env::args;
use state::{AppState, State, WindowState};
use traits::app::{AppEvent, AppCommand, AppMethods};
use traits::view::*;
use traits::window::{WindowEvent, WindowCommand};

const PKG_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const PKG_NAME: &'static str = env!("CARGO_PKG_NAME");

fn main() {

    let logs = logs::Logger::init();

    info!("starting");

    args().find(|arg| arg == "--version").map(|_| {
        println!("{} {}", PKG_NAME, PKG_VERSION);
        std::process::exit(0);
    });

    let resources_path = App::get_resources_path().expect("Can't find resources path");

    let mut app_state = State::new(AppState::new());
    app_state.get_mut().current_window_index = Some(0);

    let mut win_state = State::new(WindowState::new());

    let app = App::new(app_state.get()).expect("Can't create application");
    let win = app.new_window(win_state.get()).expect("Can't create application");
    app_state.snapshot();
    win_state.snapshot();

    let view = win.new_view().unwrap();

    Servo::configure(resources_path.clone());

    let servo = {
        let geometry = view.get_geometry();
        let waker = win.new_event_loop_waker();
        Servo::new(geometry, view.clone(), waker)
    };

    let home_url = resources_path.parent().unwrap().join("shell_resources").join("home.html");
    let home_url = ServoUrl::from_file_path(&home_url).unwrap().into_string();

    // Skip first argument (executable), and find the first
    // argument that doesn't start with `-`
    let url = args().skip(1).find(|arg| {
        !arg.starts_with("-")
    }).unwrap_or(home_url);

    let browser = servo.new_browser(&url);
    servo.select_browser(browser.id);

    win_state.get_mut().current_browser_index = Some(0);
    win_state.get_mut().browsers.push(browser);
    win.render(win_state.diff(), win_state.get());
    win_state.snapshot();

    info!("Servo version: {}", servo.version());

    let handle_events = || {

        // Loop until no events are available anymore.
        loop {

            let app_events = app.get_events();
            let win_events = win.get_events();
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
                                app_state.get_mut().dark_theme = !app_state.get().dark_theme;
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
                    WindowEvent::OptionsClosed => {
                        win_state.get_mut().options_open = false;
                    }
                    WindowEvent::UrlbarFocusChanged(focused) => {
                        let idx = win_state.get().current_browser_index.unwrap();
                        win_state.get_mut().browsers[idx].urlbar_focused = focused;
                    }
                    WindowEvent::DoCommand(cmd) => {
                        let idx = win_state.get().current_browser_index.unwrap();
                        let bid = win_state.get().browsers[idx].id;
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
                                win_state.get_mut().browsers[idx].urlbar_focused = true;
                            }
                            WindowCommand::OpenInDefaultBrowser => {
                                if let Some(ref url) = win_state.get().browsers[idx].url {
                                    open::that(url.clone()).ok();
                                }
                            }
                            WindowCommand::ZoomIn => {
                                win_state.get_mut().browsers[idx].zoom *= 1.1;
                                servo.zoom(win_state.get().browsers[idx].zoom);
                            }
                            WindowCommand::ZoomOut => {
                                win_state.get_mut().browsers[idx].zoom /= 1.1;
                                servo.zoom(win_state.get().browsers[idx].zoom);
                            }
                            WindowCommand::ZoomToActualSize => {
                                win_state.get_mut().browsers[idx].zoom = 1.0;
                                servo.reset_zoom();
                            }

                            WindowCommand::ToggleSidebar => {
                                win_state.get_mut().sidebar_is_open = !win_state.get().sidebar_is_open;
                            }

                            WindowCommand::ShowOptions => {
                                win_state.get_mut().options_open = !win_state.get().options_open;
                            }

                            WindowCommand::Load(request) => {
                                win_state.get_mut().browsers[idx].user_input = Some(request.clone());
                                win_state.get_mut().browsers[idx].urlbar_focused = false;
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
                                win_state.get_mut().logs_visible = !win_state.get().logs_visible;
                            },
                            WindowCommand::NewTab => {
                                let mut browser = servo.new_browser("about:blank");
                                if cfg!(all(not(feature = "force-glutin"), target_os = "macos")) {
                                    browser.urlbar_focused = true;
                                }
                                servo.select_browser(browser.id);
                                servo.update_geometry(view.get_geometry());
                                win_state.get_mut().current_browser_index = Some(idx + 1);
                                win_state.get_mut().browsers.push(browser);
                            },
                            WindowCommand::CloseTab => {
                                if win_state.get().browsers.len() > 1 {
                                    let id = win_state.get().browsers[idx].id;
                                    let new_id = if idx == win_state.get().browsers.len() - 1 {
                                        win_state.get_mut().current_browser_index = Some(idx - 1);
                                        win_state.get().browsers[idx - 1].id
                                    } else {
                                        win_state.get().browsers[idx + 1].id
                                    };
                                    servo.select_browser(new_id);
                                    servo.close_browser(id);
                                    win_state.get_mut().browsers.remove(idx);
                                }
                            },
                            WindowCommand::PrevTab => {
                                let new_idx = if idx == 0 {
                                    win_state.get().browsers.len() - 1
                                } else {
                                    idx - 1
                                };
                                win_state.get_mut().current_browser_index = Some(new_idx);
                                let id = win_state.get().browsers[new_idx].id;
                                servo.select_browser(id);
                            },
                            WindowCommand::NextTab => {
                                let new_idx = if idx == win_state.get().browsers.len() - 1 {
                                    0
                                } else {
                                    idx + 1
                                };
                                win_state.get_mut().current_browser_index = Some(new_idx);
                                let id = win_state.get().browsers[new_idx].id;
                                servo.select_browser(id);
                            },
                            WindowCommand::SelectTab(idx) => {
                                if win_state.get().current_browser_index != Some(idx) {
                                    win_state.get_mut().current_browser_index = Some(idx);
                                    let id = win_state.get().browsers[idx].id;
                                    servo.select_browser(id);
                                }
                            },
                            WindowCommand::ToggleOptionFragmentBorders => { },
                            WindowCommand::ToggleOptionParallelDisplayListBuidling => { },
                            WindowCommand::ToggleOptionShowParallelLayout => { },
                            WindowCommand::ToggleOptionConvertMouseToTouch => { },
                            WindowCommand::ToggleOptionTileBorders => { },

                            WindowCommand::ToggleOptionWRProfiler => {
                                win_state.get_mut().debug_options.wr_profiler = !win_state.get().debug_options.wr_profiler;
                                servo.toggle_webrender_debug_option(WebRenderDebugOption::Profiler);
                            },

                            WindowCommand::ToggleOptionWRTextureCacheDebug => {
                                win_state.get_mut().debug_options.wr_texture_cache_debug = !win_state.get().debug_options.wr_texture_cache_debug;
                                servo.toggle_webrender_debug_option(WebRenderDebugOption::TextureCacheDebug);
                            },

                            WindowCommand::ToggleOptionWRTargetDebug => {
                                win_state.get_mut().debug_options.wr_render_target_debug = !win_state.get().debug_options.wr_render_target_debug;
                                servo.toggle_webrender_debug_option(WebRenderDebugOption::RenderTargetDebug);
                            },
                        }
                    }
                }
            }

            for event in view_events {
                match event {
                    ViewEvent::GeometryDidChange => {
                        servo.update_geometry(view.get_geometry());
                        view.update_drawable();
                    }
                    ViewEvent::MouseWheel(delta, phase) => {
                        // FIXME: magic value
                        static LINE_HEIGHT: f32 = 38.0;
                        let (mut x, mut y) = match delta {
                            MouseScrollDelta::PixelDelta(x, y) => (x, y),
                            MouseScrollDelta::LineDelta(x, y) => (x, y * LINE_HEIGHT),
                        };
                        if y.abs() >= x.abs() { x = 0.0; } else { y = 0.0; }
                        servo.perform_scroll(0, 0, x, y, phase);
                    }
                    ViewEvent::MouseMoved(x, y) => {
                        servo.perform_mouse_move(x, y);
                    }
                    ViewEvent::MouseInput(element_state, button, x, y) => {
                        servo.perform_click(x, y, element_state, button);
                    }
                    ViewEvent::KeyEvent(c, key, keystate, modifiers) => {
                        let idx = win_state.get().current_browser_index.unwrap();
                        let id = win_state.get().browsers[idx].id;
                        servo.send_key(id, c, key, keystate, modifiers);
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
                        match win_state.get_mut().browsers.iter_mut().find(|b| b.id == id) {
                            Some(browser) => {
                                browser.title = title;
                            }
                            None => warn!("Got message for unkown browser:  {:?}", id)
                        }
                    }
                    ServoEvent::StatusChanged(status) => {
                        win_state.get_mut().status = status;
                    }
                    ServoEvent::LoadStart(id) => {
                        match win_state.get_mut().browsers.iter_mut().find(|b| b.id == id) {
                            Some(browser) => {
                                browser.is_loading = true;
                            }
                            None => warn!("Got message for unkown browser:  {:?}", id)
                        }
                    }
                    ServoEvent::LoadEnd(id) => {
                        match win_state.get_mut().browsers.iter_mut().find(|b| b.id == id) {
                            Some(browser) => {
                                browser.is_loading = false;
                            }
                            None => warn!("Got message for unkown browser:  {:?}", id)
                        }
                    }
                    ServoEvent::HeadParsed(..) => {
                        // FIXME
                    }
                    ServoEvent::HistoryChanged(id, entries, current) => {
                        match win_state.get_mut().browsers.iter_mut().find(|b| b.id == id) {
                            Some(browser) => {
                                let url = entries[current].url.to_string();
                                browser.url = Some(url);
                                browser.can_go_back = current > 0;
                                browser.can_go_forward = current < entries.len() - 1;
                            }
                            None => warn!("Got message for unkown browser:  {:?}", id)
                        }
                    }
                    ServoEvent::CursorChanged(cursor) => {
                        // FIXME: Work-around https://github.com/servo/servo/issues/18599
                        if cursor != app_state.get().cursor {
                            app_state.get_mut().cursor = cursor;
                        }
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

            if app_state.has_changed() || win_state.has_changed() {
                app.render(app_state.diff(), app_state.get());
                win.render(win_state.diff(), win_state.get());
                app_state.snapshot();
                win_state.snapshot();
            }


            servo.sync(force_sync);
        }

        // Here, only stuff that we know for sure won't trigger any
        // new events

        // FIXME: logs will grow until pulled
        if win_state.get().logs_visible {
            win.append_logs(&logs.get_logs());
        }
    };

    view.set_live_resize_callback(&handle_events);

    app.run(handle_events);

}
