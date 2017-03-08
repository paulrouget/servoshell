#![feature(box_syntax)]

#[macro_use]
extern crate objc;

#[macro_use]
extern crate log;

extern crate simplelog;
extern crate libc;
extern crate cocoa;
extern crate objc_foundation;

mod app;
mod controls;
mod window;
mod view;
mod servo;
mod platform;

use app::AppEvent;
use window::WindowEvent;
use view::ViewEvent;
use servo::ServoEvent;
use controls::ControlEvent;
use simplelog::{CombinedLogger, Config, LogLevel, LogLevelFilter, TermLogger, WriteLogger};
use std::fs::File;
use std::env::args;
use app::App;
use servo::{Servo, FollowLinkPolicy};

fn main() {

    // FIXME: can we use NSLog instead of a file?
    let log_file = File::create("/tmp/servoshell.log").unwrap();
    let log_config = Config {
        time: None,
        level: Some(LogLevel::Info),
        target: Some(LogLevel::Info),
        location: Some(LogLevel::Info),
    };

    let _ = match TermLogger::new(LogLevelFilter::Info, log_config) {
        Some(logger) => CombinedLogger::init(vec![
            logger,
            WriteLogger::new(LogLevelFilter::Info, log_config, log_file)
        ]),
        None => WriteLogger::init(LogLevelFilter::Info, log_config, log_file)
    };

    info!("starting");

    platform::init();

    let (app, ctrls) = App::load().unwrap();
    let (window, view) = app.create_window(&ctrls).unwrap();

    // Skip first argument (executable), and find the first
    // argument that doesn't start with `-`
    let url = args().skip(1).find(|arg| {
        !arg.starts_with("-")
    }).unwrap_or("http://servo.org".to_owned());

    Servo::configure(&url).unwrap();
    let servo = {
        let geometry = view.get_geometry();
        let riser = window.create_eventloop_riser();
        let policy = FollowLinkPolicy::FollowOriginalDomain;
        Servo::new(geometry, riser, &url, policy)
    };

    info!("Servo version: {}", servo.version());

    view.enter_fullscreen();

    app.run(|| {

        // Loop until no events are available anymore.
        loop {

            let app_events = app.get_events();
            let ctrls_events = ctrls.get_events();
            let win_events = window.get_events();
            let view_events = view.get_events();
            let servo_events = servo.get_events();

            if app_events.is_empty() &&
               ctrls_events.is_empty() &&
               win_events.is_empty() &&
               view_events.is_empty() &&
               servo_events.is_empty() {
                   break
            }

            // FIXME: it's really annoying we need this
            let mut sync_needed = false;

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
                        sync_needed = true;
                    }
                }
            }

            for event in ctrls_events {
                match event {
                    ControlEvent::Stop => {
                        // FIXME
                    }
                    ControlEvent::Reload => {
                        servo.reload();
                        sync_needed = true;
                    }
                    ControlEvent::GoBack => {
                        servo.go_back();
                        sync_needed = true;
                    }
                    ControlEvent::GoForward => {
                        servo.go_forward();
                        sync_needed = true;
                    }
                    ControlEvent::ZoomIn => {
                        // FIXME
                    }
                    ControlEvent::ZoomOut => {
                        // FIXME
                    }
                    ControlEvent::ZoomToActualSize => {
                        // FIXME
                    }
                }
            }

            for event in win_events {
                match event {
                    WindowEvent::EventLoopRised => {
                        sync_needed = true;
                    }
                    WindowEvent::GeometryDidChange => {
                        servo.update_geometry(view.get_geometry());
                        view.update_drawable();
                        sync_needed = true;
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
                }
            }

            for event in view_events {
                match event {
                    ViewEvent::MouseWheel(delta, phase) => {
                        let (x, y) = match delta {
                            view::MouseScrollDelta::PixelDelta(x, y) => {
                                (x, y)
                            },
                            _ => (0.0, 0.0),
                        };
                        servo.perform_scroll(0, 0, x, y, phase);
                        sync_needed = true;
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
                    ServoEvent::Present => {
                        view.swap_buffers();
                    }
                    ServoEvent::TitleChanged(title) => {
                        window.set_title(&title.unwrap_or("No Title".to_owned()));

                    }
                    ServoEvent::UnhandledURL(..) => {
                        // FIXME
                    }
                    ServoEvent::StatusChanged(..) => {
                        // FIXME
                    }
                    ServoEvent::LoadStart(..) => {
                        ctrls.set_command_state(ControlEvent::Reload, false);
                        ctrls.set_command_state(ControlEvent::Stop, true);
                    }
                    ServoEvent::LoadEnd(..) => {
                        ctrls.set_command_state(ControlEvent::Reload, true);
                        ctrls.set_command_state(ControlEvent::Stop, false);
                    }
                    ServoEvent::LoadError(..) => {
                        // FIXME
                    }
                    ServoEvent::HeadParsed(url) => {
                        window.set_url(url.as_str());
                    }
                    ServoEvent::CursorChanged(..) => {
                        // FIXME
                    }
                    ServoEvent::FaviconChanged(..) => {
                        // FIXME
                    }
                    ServoEvent::Key(..) => {
                        // FIXME
                    }
                }
            }

            if sync_needed {
                servo.sync();
            }
        }
    });

}
