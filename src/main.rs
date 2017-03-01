#![feature(box_syntax)]

#[macro_use]
extern crate objc;

extern crate libc;
extern crate rand;
extern crate cocoa;
extern crate objc_foundation;

mod platform;
mod servo_wrapper;
mod view_events;

use std::env::args;

use platform::app;
use servo_wrapper::{configure_servo, ServoWrapper, ServoEvent, FollowLinkPolicy};
use view_events::{ViewEvent, MouseScrollDelta};

#[derive(Copy, Clone)]
pub struct DrawableGeometry {
    pub inner_size: (u32, u32),
    pub position: (i32, i32),
    pub hidpi_factor: f32,
}

fn main() {

    app::load().unwrap();

    let view = app::new_window().unwrap();

    let url = args().nth(1).unwrap_or("http://servo.org".to_owned());

    configure_servo(&url);

    let servo = {
        let geometry = view.get_geometry();
        let riser = view.create_eventloop_riser();
        let policy = FollowLinkPolicy::FollowOriginalDomain;
        ServoWrapper::new(geometry, riser, &url, policy)
    };

    app::run(|| {

        let mut sync_needed = false;
        let mut swap_buffers_needed = false;

        for e in view.get_events().into_iter() {
            match e {
                ViewEvent::Refresh | ViewEvent::Awakened => {
                    swap_buffers_needed = true;
                    sync_needed = true;
                }
                ViewEvent::MouseWheel(delta, phase) => {
                    use self::MouseScrollDelta::PixelDelta;
                    let (dx, dy) = match delta {
                        PixelDelta(dx, dy) => (dx, dy),
                        _ => (0.0, 0.0) // FIXME
                    };
                    servo.perform_scroll(0,0,dx,dy,phase);
                    sync_needed = true;
                }
                _ => { }
            }
        }

        for e in servo.get_events().into_iter() {
            match e {
                ServoEvent::Present => {
                    swap_buffers_needed = true;
                }
                _ => {
                }
            }
        }

        if swap_buffers_needed {
            view.swap_buffers();
        }

        if sync_needed {
            servo.sync();
        }
    });

}
