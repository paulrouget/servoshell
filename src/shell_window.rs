use glutin;
use gleam::gl;
use servo::compositing::windowing::WindowMethods;
use servo::compositing::compositor_thread::{self, CompositorProxy, CompositorReceiver};
use servo::msg::constellation_msg::{self, Key};
use servo::euclid::{Point2D, Size2D};
use servo::euclid::scale_factor::ScaleFactor;
use servo::euclid::size::TypedSize2D;
use servo::script_traits::DevicePixel;
use servo::servo_url::ServoUrl;
use servo::net_traits::net_error_list::NetError;
use servo_geometry::ScreenPx;
use style_traits::cursor::Cursor;
use std::os::raw::c_void;
use std::sync::mpsc::{Sender, channel};

use winit::os::macos::WindowExt;
use cocoa::base::*;
use cocoa::foundation::*;
use cocoa::appkit::*;
use objc::runtime::YES;

use ui::build_ui;

struct GlutinCompositorProxy {
    sender: Sender<compositor_thread::Msg>,
    window_proxy: Option<glutin::WindowProxy>,
}

impl CompositorProxy for GlutinCompositorProxy {
    fn send(&self, msg: compositor_thread::Msg) {
        // Send a message and kick the OS event loop awake.
        if let Err(err) = self.sender.send(msg) {
            println!("Failed to send response ({}).", err);
        }
        if let Some(ref window_proxy) = self.window_proxy {
            window_proxy.wakeup_event_loop()
        }
    }

    fn clone_compositor_proxy(&self) -> Box<CompositorProxy + Send> {
        box GlutinCompositorProxy {
            sender: self.sender.clone(),
            window_proxy: self.window_proxy.clone(),
        } as Box<CompositorProxy + Send>
    }
}

pub struct ShellWindow {
    glutin_window: glutin::Window,
}

impl ShellWindow {
    pub fn new() -> ShellWindow {
        let builder = glutin::WindowBuilder::new().
            with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2))).
            // with_decorations(false).
            with_dimensions(800, 600).
            with_vsync();
        let glutin_window = builder.build().expect("Failed to create window.");

        unsafe {
            glutin_window.make_current().expect("Couldn't make window current");
            gl::load_with(|s| glutin_window.get_proc_address(s) as *const c_void);
            gl::clear_color(1.0, 0.0, 0.0, 1.0);
            gl::clear(gl::COLOR_BUFFER_BIT);
            gl::finish();
        }

        let nsview = glutin_window.as_winit_window().get_nsview() as id;
        let nswindow = glutin_window.as_winit_window().get_nswindow() as id;

        unsafe {
            build_ui(nswindow);
        }

        ShellWindow {
            glutin_window: glutin_window
        }
    }

    pub fn glutin_window(&self) -> &glutin::Window {
        &self.glutin_window
    }
}

impl WindowMethods for ShellWindow {
    fn framebuffer_size(&self) -> TypedSize2D<u32, DevicePixel> {
        let scale_factor = self.glutin_window.hidpi_factor() as u32;
        let (width, height) = self.glutin_window.get_inner_size().expect("Failed to get window inner size.");
        TypedSize2D::new(scale_factor * width, scale_factor * height)
    }

    fn size(&self) -> TypedSize2D<f32, ScreenPx> {
        let (width, height) = self.glutin_window.get_inner_size().expect("Failed to get window inner size.");
        TypedSize2D::new(width as f32, height as f32)
    }

    fn client_window(&self) -> (Size2D<u32>, Point2D<i32>) {
        let (width, height) = self.glutin_window.get_inner_size().expect("Failed to get window inner size.");
        let size = Size2D::new(width, height);
        let (x, y) = self.glutin_window.get_position().expect("Failed to get window position.");
        let origin = Point2D::new(x as i32, y as i32);
        (size, origin)
    }

    fn set_inner_size(&self, size: Size2D<u32>) {
        self.glutin_window.set_inner_size(size.width as u32, size.height as u32)
    }

    fn set_position(&self, point: Point2D<i32>) {
        self.glutin_window.set_position(point.x, point.y)
    }

    fn set_fullscreen_state(&self, _state: bool) {
    }

    fn present(&self) {
        self.glutin_window.swap_buffers().expect("swap_buffers failed");
    }

    fn create_compositor_channel(&self) -> (Box<CompositorProxy + Send>, Box<CompositorReceiver>) {
        let (sender, receiver) = channel();
        (box GlutinCompositorProxy {
             sender: sender,
             window_proxy: Some(self.glutin_window.create_window_proxy())
         } as Box<CompositorProxy + Send>,
         box receiver as Box<CompositorReceiver>)
    }

    fn scale_factor(&self) -> ScaleFactor<f32, ScreenPx, DevicePixel> {
        ScaleFactor::new(self.glutin_window.hidpi_factor())
    }

    fn set_page_title(&self, _title: Option<String>) {
    }

    fn set_page_url(&self, _url: ServoUrl) {
    }

    fn status(&self, _: Option<String>) {
    }

    fn load_start(&self, _: bool, _: bool) {
    }

    fn load_end(&self, _: bool, _: bool, _root: bool) {
    }

    fn load_error(&self, _: NetError, _: String) {
    }

    fn head_parsed(&self) {
    }

    fn set_cursor(&self, _: Cursor) {
    }

    fn set_favicon(&self, _: ServoUrl) {
    }

    fn prepare_for_composite(&self, _width: usize, _height: usize) -> bool {
        true
    }

    fn handle_key(&self, _ch: Option<char>, _key: Key, _mods: constellation_msg::KeyModifiers) {
    }

    fn supports_clipboard(&self) -> bool {
        false
    }
}
