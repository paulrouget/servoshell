#![feature(box_syntax)]

// Copy necessary for Cell<DrawableGeometry>. Clone is necessary for Copy.
// FIXME: I'm not sure why.
#[derive(Copy, Clone)]
pub struct DrawableGeometry {
    inner_size: (u32, u32),
    position: (i32, i32),
    hidpi_factor: f32,
}

// FIXME: can I move that to chrome.rs???
#[macro_use]
extern crate objc;

extern crate servo;
extern crate winit;

const LINE_HEIGHT: f32 = 38.0; // FIXME

use servo::servo_url::ServoUrl;

mod window;
mod platform;
mod browser;

fn get_url() -> ServoUrl {
    let default_url = ServoUrl::parse("http://servo.org").ok();
    match std::env::args().nth(1) {
        None => default_url,
        Some(arg1) => {
            match ServoUrl::parse(arg1.as_str()) {
                Err(_) => default_url,
                Ok(url) => Some(url)
            }
        }
    }.unwrap()
}


fn main() {

    let window = window::ChromeWindow::new();

    let widgets = platform::implementation::Widgets::new(&window);

    let browser = browser::Browser::new(
        window.get_geometry(),
        window.create_window_proxy(),
        get_url());

    browser.initialize_compositing();

    let mut mouse_pos = (0, 0);
    let mut mouse_down_button: Option<winit::MouseButton> = None;
    let mut mouse_down_point = (0, 0);

    loop {
        let mut winit_events = window.get_events();
        let mut widget_events = widgets.get_events();
        let mut browser_events = browser.get_events();

        for event in widget_events.drain(..) {
            match event {
                e => {
                    // FIXME: next
                    browser.reload();
                    println!("widget event: {:?}", e);
                }
            }
        }

        for event in browser_events.drain(..) {
            match event {
                // FIXME: rename ServoEvent to BrowserEvent
                browser::ServoEvent::Present => {
                    window.swap_buffers();
                }
                browser::ServoEvent::TitleChanged(title) => {
                    match title {
                        None => widgets.set_urlbar_text(""),
                        Some(text) => widgets.set_urlbar_text(text.as_str()),
                    }
                }
                _ => {
                }
            }
        }

        for event in winit_events.drain(..) {
            match event {
                winit::Event::MouseMoved(x, y) => {
                    let y = y - 76; /* FIXME: magic value */
                    mouse_pos = (x, y);
                    browser.update_mouse_coordinates(x, y);
                }
                winit::Event::MouseWheel(delta, phase) => {
                    let (mut dx, mut dy) = match delta {
                        winit::MouseScrollDelta::LineDelta(dx, dy) => (dx, dy * LINE_HEIGHT),
                        winit::MouseScrollDelta::PixelDelta(dx, dy) => (dx, dy),
                    };
                    if dy.abs() >= dx.abs() {
                        dx = 0.0;
                    } else {
                        dy = 0.0;
                    }
                    let (x, y) = mouse_pos;
                    browser.scroll(x, y, dx, dy, phase);
                }
                _ => { }
            }
        }

        // FIXME: Is there a cleaner way to achieve this?
        // sync is necessary event if there's no event
        // The main thread is awaken by Servo (see CompositorProxy trick)
        // servo.handle_event() is expected to be called.
        browser.sync();
    }
}
