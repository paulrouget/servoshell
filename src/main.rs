#![feature(box_syntax)]

#[macro_use]
extern crate objc;

extern crate libc;
extern crate rand;
extern crate cocoa;
extern crate objc_foundation;

mod app;
mod window;
mod view;
mod servo;
mod platform;

use app::AppEvent;
use window::WindowEvent;
use view::ViewEvent;
use servo::ServoEvent;

use std::env::args;

use app::App;
use servo::{Servo, FollowLinkPolicy};

fn main() {

    let app = App::new().unwrap();
    let (window, view) = app.create_window().unwrap();

    let url = args().nth(1).unwrap_or("http://servo.org".to_owned());
    Servo::configure(&url);
    let servo = {
        let geometry = view.get_geometry();
        let riser = window.create_eventloop_riser();
        let policy = FollowLinkPolicy::FollowOriginalDomain;
        Servo::new(geometry, riser, &url, policy)
    };

    println!("Servo version: {}", servo.version());

    app.run(|| {

        let app_events = app.get_events();
        let win_events = window.get_events();
        let view_events = view.get_events();
        let servo_events = servo.get_events();

        // if !app_events.is_empty() {
        //     println!("app_events: {:?}", app_events);
        // }
        // if !win_events.is_empty() {
        //     println!("win_events: {:?}", win_events);
        // }
        // if !view_events.is_empty() {
        //     println!("view_events: {:?}", view_events);
        // }
        // if !servo_events.is_empty() {
        //     println!("servo_events: {:?}", servo_events);
        // }

        let mut sync_needed = false;

        for event in win_events {
            match event {
                WindowEvent::ReloadClicked => {
                    println!("Yep. Reload clicked.");
                }
                WindowEvent::EventLoopRised => {
                    sync_needed = true;
                }
                WindowEvent::GeometryDidChange => {
                    servo.update_geometry(view.get_geometry());
                    view.update_drawable();
                    sync_needed = true;
                }
                _ => { }
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
                _ => { }
            }
        }

        for event in servo_events {
            match event {
                ServoEvent::Present => {
                    view.swap_buffers();
                }
                _ => { }
            }
        }

        if sync_needed {
            servo.sync();
        }
    });

}
