#![feature(box_syntax)]

// It's necessary to declare this here:
// rustc: "an `extern crate` loading macros must be at the crate root"
#[macro_use]
extern crate objc;

mod window;
mod widgets;
mod servo;

use std::env::args;
use servo::{Servo, ServoEvent, ServoCursor};
use window::{GlutinWindow, WindowEvent, WindowMouseButton, WindowMouseCursor};
use window::{WindowMouseScrollDelta, WindowElementState};

#[derive(Copy, Clone)]
pub struct DrawableGeometry {
    inner_size: (u32, u32),
    position: (i32, i32),
    hidpi_factor: f32,
}

fn main() {

    let url = args().nth(1).unwrap_or("http://servo.org".to_owned());
    let window = GlutinWindow::new();
    let widgets = widgets::platform::Widgets::new(&window);
    let servo = Servo::new(window.get_geometry(), window.create_event_loop_riser(), &url);

    let mut mouse_pos = (0, 0);
    let mut mouse_down_button: Option<WindowMouseButton> = None;
    let mut mouse_down_point = (0, 0);

    loop {
        let mut winit_events = window.get_events();
        let mut widget_events = widgets.get_events();
        let mut browser_events = servo.get_events();

        for event in widget_events.drain(..) {
            match event {
                widgets::WidgetEvent::BackClicked => {
                    servo.go_back();
                }
                widgets::WidgetEvent::FwdClicked => {
                    servo.go_fwd();
                }
                widgets::WidgetEvent::ReloadClicked => {
                    servo.reload();
                }
            }
        }

        for event in browser_events.drain(..) {
            match event {
                ServoEvent::CursorChanged(servo_cursor) => {
                    let winit_cursor = match servo_cursor {
                        ServoCursor::None => WindowMouseCursor::NoneCursor,
                        ServoCursor::Default => WindowMouseCursor::Default,
                        ServoCursor::Pointer => WindowMouseCursor::Hand,
                        ServoCursor::ContextMenu => WindowMouseCursor::ContextMenu,
                        ServoCursor::Help => WindowMouseCursor::Help,
                        ServoCursor::Progress => WindowMouseCursor::Progress,
                        ServoCursor::Wait => WindowMouseCursor::Wait,
                        ServoCursor::Cell => WindowMouseCursor::Cell,
                        ServoCursor::Crosshair => WindowMouseCursor::Crosshair,
                        ServoCursor::Text => WindowMouseCursor::Text,
                        ServoCursor::VerticalText => WindowMouseCursor::VerticalText,
                        ServoCursor::Alias => WindowMouseCursor::Alias,
                        ServoCursor::Copy => WindowMouseCursor::Copy,
                        ServoCursor::Move => WindowMouseCursor::Move,
                        ServoCursor::NoDrop => WindowMouseCursor::NoDrop,
                        ServoCursor::NotAllowed => WindowMouseCursor::NotAllowed,
                        ServoCursor::Grab => WindowMouseCursor::Grab,
                        ServoCursor::Grabbing => WindowMouseCursor::Grabbing,
                        ServoCursor::EResize => WindowMouseCursor::EResize,
                        ServoCursor::NResize => WindowMouseCursor::NResize,
                        ServoCursor::NeResize => WindowMouseCursor::NeResize,
                        ServoCursor::NwResize => WindowMouseCursor::NwResize,
                        ServoCursor::SResize => WindowMouseCursor::SResize,
                        ServoCursor::SeResize => WindowMouseCursor::SeResize,
                        ServoCursor::SwResize => WindowMouseCursor::SwResize,
                        ServoCursor::WResize => WindowMouseCursor::WResize,
                        ServoCursor::EwResize => WindowMouseCursor::EwResize,
                        ServoCursor::NsResize => WindowMouseCursor::NsResize,
                        ServoCursor::NeswResize => WindowMouseCursor::NeswResize,
                        ServoCursor::NwseResize => WindowMouseCursor::NwseResize,
                        ServoCursor::ColResize => WindowMouseCursor::ColResize,
                        ServoCursor::RowResize => WindowMouseCursor::RowResize,
                        ServoCursor::AllScroll => WindowMouseCursor::AllScroll,
                        ServoCursor::ZoomIn => WindowMouseCursor::ZoomIn,
                        ServoCursor::ZoomOut => WindowMouseCursor::ZoomOut,
                    };
                    window.get_winit_window().set_cursor(winit_cursor);
                }
                ServoEvent::Present => {
                    window.swap_buffers();
                }
                ServoEvent::LoadStart(_, _) => {
                    widgets.set_indicator_active(true);
                }
                ServoEvent::LoadEnd(_, _, _) => {
                    widgets.set_indicator_active(false);
                }
                ServoEvent::TitleChanged(title) => {
                    match title {
                        None => widgets.set_urlbar_text(""),
                        Some(text) => widgets.set_urlbar_text(text.as_str()),
                    }
                }
                e => {
                    println!("Unhandled Servo event: {:?}", e);
                }
            }
        }

        for event in winit_events.drain(..) {
            match event {
                WindowEvent::MouseMoved(x, y) => {
                    let y = y - 76; /* FIXME: magic value */
                    mouse_pos = (x, y);
                    servo.update_mouse_coordinates(x, y);
                }
                WindowEvent::MouseWheel(delta, phase) => {
                    let (mut dx, mut dy) = match delta {
                        // FIXME: magic value
                        WindowMouseScrollDelta::LineDelta(dx, dy) => (dx, dy * 38.),
                        WindowMouseScrollDelta::PixelDelta(dx, dy) => (dx, dy),
                    };
                    if dy.abs() >= dx.abs() {
                        dx = 0.0;
                    } else {
                        dy = 0.0;
                    }
                    let (x, y) = mouse_pos;
                    servo.scroll(x, y, dx, dy, phase);
                }
                WindowEvent::MouseInput(element_state, mouse_button) => {
                    if mouse_button == WindowMouseButton::Left ||
                       mouse_button == WindowMouseButton::Right {
                        if element_state == WindowElementState::Pressed {
                            mouse_down_point = mouse_pos;
                            mouse_down_button = Some(mouse_button);
                        }
                        let (x, y) = mouse_pos;
                        let (org_x, org_y) = mouse_down_point;
                        servo.click(x,
                                    y,
                                    org_x,
                                    org_y,
                                    element_state,
                                    mouse_button,
                                    mouse_down_button);
                    }
                }
                _ => {}
            }
        }

        // sync is necessary even if there's no event.
        // The main thread is awaken by Servo (see CompositorProxy trick).
        // servo.handle_event() is then expected to be called.
        servo.sync();
    }
}
