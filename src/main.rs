#![feature(box_syntax)]

extern crate glutin;
extern crate winit;
extern crate cocoa;
extern crate gleam;
#[macro_use]
extern crate objc;
extern crate servo;
extern crate servo_geometry;
extern crate style_traits;
extern crate euclid;
extern crate webrender_traits;

mod ui;
mod shell_window;
mod events;

use events::GlutinEventHandler;
use std::rc::Rc;
use servo::config::servo_version;
use servo::compositing::windowing::WindowEvent;
use servo::servo_config::opts;
use servo::servo_config::prefs::{PrefValue, PREFS};
use servo::servo_url::ServoUrl;
use shell_window::ShellWindow;

fn main() {
    println!("{}", servo_version());

    let mut opts = opts::default_opts();
    opts.headless = false;
    opts.url = ServoUrl::parse(std::env::args().nth(1).unwrap().as_str()).ok();
    opts::set_defaults(opts);

    // Pipeline creation fails is layout_threads pref not set
    PREFS.set("layout.threads", PrefValue::Number(1.0));

    let shell_window = Rc::new(ShellWindow::new());

    let glutin_event_handler = GlutinEventHandler::new();

    let mut browser = servo::Browser::new(shell_window.clone());
    browser.handle_events(vec![WindowEvent::InitializeCompositing]);

    loop {
        let glutin_event = shell_window.glutin_window().wait_events().next();
        let closed = glutin_event_handler.handle_glutin_event(glutin_event.unwrap());
        if closed {
            // FIXME
        }
        let servo_events = glutin_event_handler.get_events_for_servo();
        browser.handle_events(servo_events);
    }
}
