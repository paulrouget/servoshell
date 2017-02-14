extern crate gleam;
extern crate winit;
extern crate glutin;

use std;
use DrawableGeometry;


pub struct ChromeWindow {
    glutin_window: glutin::Window,
}

impl ChromeWindow {
    pub fn new() -> ChromeWindow {

        let glutin_window = glutin::WindowBuilder::new()
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)))
            .with_dimensions(800, 600)
            .with_vsync()
            .build().expect("Failed to create window.");

        unsafe {
            glutin_window.make_current().expect("Couldn't make window current");
            gleam::gl::load_with(|s| {
                glutin_window.get_proc_address(s) as *const std::os::raw::c_void
            });
            gleam::gl::clear_color(1.0, 0.0, 0.0, 1.0);
            gleam::gl::clear(gleam::gl::COLOR_BUFFER_BIT);
            gleam::gl::finish();
        }

        ChromeWindow {
            glutin_window: glutin_window
        }
    }

    pub fn get_geometry(&self) -> DrawableGeometry {
        DrawableGeometry {
            inner_size: self.glutin_window.get_inner_size().expect("Failed to get window inner size."),
            position: self.glutin_window.get_position().expect("Failed to get window position."),
            hidpi_factor: self.glutin_window.hidpi_factor(),
        }
    }

    pub fn create_window_proxy(&self) -> glutin::WindowProxy {
        self.glutin_window.create_window_proxy()
    }

    pub fn get_winit_window(&self) -> &winit::Window {
        self.glutin_window.as_winit_window()
    }

    // FIXME: can we have a separate function to wait until wake up?
    pub fn get_events(&self) -> Vec<winit::Event> {
        let mut events: Vec<winit::Event> = Vec::new();
        let event = self.glutin_window.wait_events().next();
        events.push(event.unwrap());
        while let Some(event) = self.glutin_window.poll_events().next() {
            events.push(event);
        }
        events
    }

    pub fn swap_buffers(&self) {
        self.glutin_window.swap_buffers().expect("Failed to swap buffers");
    }
}
