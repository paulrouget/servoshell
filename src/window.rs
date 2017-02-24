extern crate gleam;
extern crate winit;
extern crate glutin;

use DrawableGeometry;

pub struct GlutinWindow {
    glutin_window: glutin::Window,
}

// Alias some enums
pub use self::winit::Event as WindowEvent;
pub use self::winit::MouseButton as WindowMouseButton;
pub use self::winit::MouseCursor as WindowMouseCursor;
pub use self::winit::MouseScrollDelta as WindowMouseScrollDelta;
pub use self::winit::ElementState as WindowElementState;
pub use self::winit::TouchPhase as WindowTouchPhase;
pub use self::winit::VirtualKeyCode as WindowVirtualKeyCode;
// FIXME: Can we avoid that?
pub use self::winit::os::macos::WindowExt;

impl GlutinWindow {
    pub fn new() -> GlutinWindow {

        let glutin_window = glutin::WindowBuilder::new()
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)))
            .with_dimensions(800, 600)
            .with_vsync()
            .build()
            .expect("Failed to create window.");


            glutin_window.make_current().expect("Couldn't make window current");
            gleam::gl::load_with(|s| glutin_window.get_proc_address(s) as *const c_void);
            gleam::gl::clear_color(1.0, 0.0, 0.0, 1.0);
            gleam::gl::clear(gleam::gl::COLOR_BUFFER_BIT);
            gleam::gl::finish();
        }

        GlutinWindow { glutin_window: glutin_window }
    }

    pub fn create_event_loop_riser(&self) -> EventLoopRiser {
        EventLoopRiser { window_proxy: self.glutin_window.create_window_proxy() }
    }

    pub fn get_geometry(&self) -> DrawableGeometry {
        // FIXME: we are assuming that the drawable region is full window.
        // As of now, it's the case. But eventually, we want to be able to draw
        // in a nsview.
        DrawableGeometry {
            inner_size: self.glutin_window
                .get_inner_size()
                .expect("Failed to get window inner size."),
            position: self.glutin_window.get_position().expect("Failed to get window position."),
            hidpi_factor: self.glutin_window.hidpi_factor(),
        }
    }

    pub fn get_winit_window(&self) -> &winit::Window {
        self.glutin_window.as_winit_window()
    }

    // FIXME: can we have a separate function to wait until wake up?
    pub fn get_events(&self) -> Vec<WindowEvent> {
        let mut events: Vec<WindowEvent> = Vec::new();
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

// Used by Servo to wake up the event loop
pub struct EventLoopRiser {
    window_proxy: winit::WindowProxy,
}

impl EventLoopRiser {
    pub fn rise(&self) {
        self.window_proxy.wakeup_event_loop()
    }
    pub fn clone(&self) -> EventLoopRiser {
        EventLoopRiser { window_proxy: self.window_proxy.clone() }
    }
}
