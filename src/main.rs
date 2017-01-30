#![feature(box_syntax)]

extern crate glutin;
extern crate winit;
extern crate cocoa;
extern crate gleam;
extern crate servo;
extern crate servo_geometry;
extern crate style_traits;

mod my_window;

use std::rc::Rc;
use servo::config::servo_version;
use servo::compositing::windowing::WindowEvent;
use servo::servo_config::opts;
use servo::servo_config::prefs::{PrefValue, PREFS};
use servo::servo_url::ServoUrl;
use my_window::MyWindow;

// FIXME: resources dir is necessary
fn main() {
    println!("{}", servo_version());

    let mut opts = opts::default_opts();
    opts.headless = false;
    opts.url = Some(ServoUrl::parse("http://paulrouget.com").unwrap());
    opts::set_defaults(opts);

    // Pipeline creation fails is layout_threads pref not set
    PREFS.set("layout.threads", PrefValue::Number(1.0));

    let w = Rc::new(MyWindow::new());

    let mut browser = servo::Browser::new(w.clone());
    browser.handle_events(vec![WindowEvent::InitializeCompositing]);
    loop {
        w.glutin_window().wait_events().next();
        // FIXME: translate glutin event to Servo event
        // let glutin_event = w.glutin_window.wait_events().next();
        // match glutin_event {
        // }
        browser.handle_events(vec![WindowEvent::Refresh]);
    }
}
