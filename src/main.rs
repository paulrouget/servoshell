#![feature(box_syntax)]

#[macro_use]
extern crate objc;

#[macro_use]
extern crate log;

extern crate simplelog;
extern crate libc;
extern crate cocoa;
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
use simplelog::{Config, LogLevel, LogLevelFilter, WriteLogger};
use std::fs::File;
use std::env::args;
use servo::{Servo, ServoUrl, FollowLinkPolicy};

use platform::get_state;

fn main() {

    // FIXME: can we use NSLog instead of a file?
    let log_file = File::create("/tmp/servoshell.log").unwrap();
    let log_config = Config {
        time: None,
        level: Some(LogLevel::Info),
        target: Some(LogLevel::Info),
        location: Some(LogLevel::Info),
    };
    let _ = WriteLogger::init(LogLevelFilter::Info, log_config, log_file);

    info!("starting");

    platform::init();

    let app = App::load().unwrap();
    let window = app.create_window().unwrap();

    get_state().current_window_index = Some(0);
    get_state().window_states.push(Window::get_init_state());

    let view = window.create_view().unwrap();

    // Skip first argument (executable), and find the first
    // argument that doesn't start with `-`
    let url = args().skip(1).find(|arg| {
        !arg.starts_with("-")
    }).unwrap_or("http://servo.org".to_owned());

    Servo::configure(&url).unwrap();
    let servo = {
        let geometry = view.get_geometry();
        let riser = window.create_eventloop_riser();
        // let policy = FollowLinkPolicy::FollowOriginalDomain;
        let policy = FollowLinkPolicy::FollowAnyLink;
        Servo::new(geometry, riser, &url, policy)
    };

    get_state().window_states[0].current_browser_index = Some(0);
    get_state().window_states[0].browser_states.push(Servo::get_init_state());

    info!("Servo version: {}", servo.version());

    app.run(|| {

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
                                app.update_theme();
                            }
                        }
                    }
                }
            }

            for event in win_events {
                match event {
                    WindowEvent::EventLoopRised => {
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
                        let ref mut state = get_state().window_states[0].browser_states[0];
                        match cmd {
                            WindowCommand::Stop => {
                                // FIXME
                            }
                            WindowCommand::Reload => {
                                servo.reload();
                            }
                            WindowCommand::NavigateBack => {
                                servo.go_back();
                            }
                            WindowCommand::NavigateForward => {
                                servo.go_forward();
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
                                        servo.load_url(url)
                                    },
                                    Err(err) => warn!("Can't parse url: {}", err),
                                }
                            }
                            WindowCommand::ToggleOptionShowLogs => { },
                            WindowCommand::ToggleOptionLockDomain => { },
                            WindowCommand::ToggleOptionFragmentBorders => { },
                            WindowCommand::ToggleOptionParallelDisplayListBuidling => { },
                            WindowCommand::ToggleOptionShowParallelLayout => { },
                            WindowCommand::ToggleOptionConvertMouseToTouch => { },
                            WindowCommand::ToggleOptionCompositorBorders => { },
                            WindowCommand::ToggleOptionShowParallelPaint => { },
                            WindowCommand::ToggleOptionPaintFlashing => { },
                            WindowCommand::ToggleOptionWebRenderStats => { },
                            WindowCommand::ToggleOptionMultisampleAntialiasing => { },
                            WindowCommand::ToggleOptionTileBorders => { },
                        }
                    }
                }
            }

            for event in view_events {
                let ref mut state = get_state().window_states[0].browser_states[0];
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
                let ref mut state = get_state().window_states[0].browser_states[0];
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
                    ServoEvent::Present => {
                        view.swap_buffers();
                    }
                    ServoEvent::TitleChanged(title) => {
                        window.set_title(&title.unwrap_or("No Title".to_owned()));

                    }
                    ServoEvent::UnhandledURL(url) => {
                        open::that(url.as_str()).ok();

                    }
                    ServoEvent::StatusChanged(..) => {
                        // FIXME
                    }
                    ServoEvent::LoadStart(can_go_back, can_go_forward) => {
                        // FIXME: See https://github.com/servo/servo/issues/15643
                        ui_invalidated = true;
                        state.is_loading = true;
                        state.can_go_back = can_go_back;
                        state.can_go_forward = can_go_forward;
                    }
                    ServoEvent::LoadEnd(can_go_back, can_go_forward, root) => {
                        // FIXME: See https://github.com/servo/servo/issues/15643
                        ui_invalidated = true;
                        state.is_loading = false;
                        if root {
                            state.can_go_back = can_go_back;
                            state.can_go_forward = can_go_forward;
                        }
                    }
                    ServoEvent::LoadError(..) => {
                        // FIXME
                    }
                    ServoEvent::HeadParsed(url) => {
                        ui_invalidated = true;
                        window.set_url(url.as_str());
                        state.url = Some(url.into_string());
                    }
                    ServoEvent::CursorChanged(cursor) => {
                        window.set_cursor(cursor);
                    }
                    ServoEvent::FaviconChanged(..) => {
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
    });

}
