extern crate servo;
extern crate servo_geometry;
extern crate style_traits;
extern crate euclid;

extern crate webrender_traits;

// FIXME: sad
extern crate winit;

use DrawableGeometry;
use std;
use self::servo::config::servo_version;
use self::servo::servo_config::opts;
use self::servo::servo_config::prefs::{PrefValue, PREFS};
use self::servo::compositing::windowing::{WindowMethods, WindowEvent};
use self::servo::compositing::compositor_thread::{self, CompositorProxy, CompositorReceiver};
use self::servo::msg::constellation_msg::{self, Key};
use self::servo::euclid::{TypedPoint2D, Point2D, Size2D};
use self::servo::euclid::scale_factor::ScaleFactor;
use self::servo::euclid::size::TypedSize2D;
use self::servo::script_traits::DevicePixel;
use self::servo::servo_url::ServoUrl;
use self::servo::net_traits::net_error_list::NetError;
use self::servo_geometry::ScreenPx;
use self::style_traits::cursor::Cursor;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use servo::script_traits::TouchEventType;

pub type CompositorChannel = (Box<compositor_thread::CompositorProxy + Send>,
                              Box<compositor_thread::CompositorReceiver>);

#[derive(Clone)]
pub enum ServoEvent {
    SetWindowInnerSize(u32, u32),
    SetWindowPosition(i32, i32),
    SetFullScreenState(bool),
    Present,
    TitleChanged(Option<String>),
    URLChanged(ServoUrl),
    StatusChanged(Option<String>),
    LoadStart(bool, bool),
    LoadEnd(bool, bool, bool),
    LoadError(String),
    HeadParsed,
    CursorChanged(Cursor),
    FaviconChanged(ServoUrl),
    Key(Option<char>, Key, constellation_msg::KeyModifiers),
}

impl std::fmt::Debug for ServoEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ServoEvent::SetWindowInnerSize(_, _) => write!(f, "SetWindowInnerSize"),
            ServoEvent::SetWindowPosition(_, _) => write!(f, "SetWindowPosition"),
            ServoEvent::SetFullScreenState(_) => write!(f, "SetFullScreenState"),
            ServoEvent::Present => write!(f, "Present"),
            ServoEvent::TitleChanged(_) => write!(f, "TitleChanged"),
            ServoEvent::URLChanged(_) => write!(f, "URLChanged"),
            ServoEvent::StatusChanged(_) => write!(f, "StatusChanged"),
            ServoEvent::LoadStart(_, _) => write!(f, "LoadStart"),
            ServoEvent::LoadEnd(_, _, _) => write!(f, "LoadEnd"),
            ServoEvent::LoadError(_) => write!(f, "LoadError"),
            ServoEvent::HeadParsed => write!(f, "HeadParsed"),
            ServoEvent::CursorChanged(_) => write!(f, "CursorChanged"),
            ServoEvent::FaviconChanged(_) => write!(f, "FaviconChanged"),
            ServoEvent::Key(_, _, _) => write!(f, "Key"),
        }
    }
}

pub struct Browser {
    events_for_servo: RefCell<Vec<WindowEvent>>,
    servo_browser: RefCell<servo::Browser<ServoCallbacks>>,
    callbacks: Rc<ServoCallbacks>,
}

impl Browser {
    pub fn new(geometry: DrawableGeometry, window_proxy: winit::WindowProxy, url: ServoUrl) -> Browser {

        let mut opts = opts::default_opts();
        opts.headless = false;
        opts.url = Some(url);
        opts::set_defaults(opts);
        PREFS.set("layout.threads", PrefValue::Number(1.0)); // FIXME: Pipeline creation fails is layout_threads pref not set

        let callbacks = Rc::new(ServoCallbacks {
            event_queue: RefCell::new(Vec::new()),
            geometry: Cell::new(geometry),
            window_proxy: window_proxy,
        });

        println!("{}", servo_version());

        let mut servo = servo::Browser::new(callbacks.clone());

        Browser {
            events_for_servo: RefCell::new(Vec::new()),
            servo_browser: RefCell::new(servo),
            callbacks: callbacks,
        }
    }

    pub fn update_geometry(&self, geometry: DrawableGeometry) {
        // FIXME
        self.callbacks.set_geometry(geometry);
    }

    pub fn get_events(&self) -> Vec<ServoEvent> {
        self.callbacks.get_events()
    }

    pub fn initialize_compositing(&self) {
        self.servo_browser.borrow_mut().handle_events(vec![WindowEvent::InitializeCompositing]);
    }

    pub fn update_mouse_coordinates(&self, x: i32, y: i32) {
        let event = WindowEvent::MouseWindowMoveEventClass(TypedPoint2D::new(x as f32, y as f32));
        self.events_for_servo.borrow_mut().push(event);
    }

    pub fn reload(&self) {
        let event = WindowEvent::Reload;
        self.events_for_servo.borrow_mut().push(event);
    }

    pub fn scroll(&self, x: i32, y: i32, dx: f32, dy: f32, phase: winit::TouchPhase) {
        let scroll_location = webrender_traits::ScrollLocation::Delta(TypedPoint2D::new(dx, dy));
        let phase = match phase {
            winit::TouchPhase::Started => TouchEventType::Down,
            winit::TouchPhase::Moved => TouchEventType::Move,
            winit::TouchPhase::Ended => TouchEventType::Up,
            winit::TouchPhase::Cancelled => TouchEventType::Cancel,
        };
        let event = WindowEvent::Scroll(scroll_location, TypedPoint2D::new(x, y), phase);
        self.events_for_servo.borrow_mut().push(event);
    }

    pub fn sync(&self) {
        let clone = self.events_for_servo.borrow().clone();
        self.events_for_servo.borrow_mut().clear();
        self.servo_browser.borrow_mut().handle_events(clone);
    }
}

struct ServoCallbacks {
    // FIXME: interior mutability - is using RefCell the right thing to do?
    event_queue: RefCell<Vec<ServoEvent>>,
    geometry: Cell<DrawableGeometry>,
    window_proxy: winit::WindowProxy,
}

impl ServoCallbacks {
    fn get_events(&self) -> Vec<ServoEvent> {
        let clone = self.event_queue.borrow().clone();
        self.event_queue.borrow_mut().clear();
        clone
    }
    fn set_geometry(&self, geometry: DrawableGeometry) {
        self.geometry.set(geometry);
    }
}

impl WindowMethods for ServoCallbacks {

    fn prepare_for_composite(&self, _width: usize, _height: usize) -> bool {
        true
    }

    fn supports_clipboard(&self) -> bool {
        // FIXME
        false
    }

    fn create_compositor_channel(&self) -> CompositorChannel {
        let (sender, receiver) = std::sync::mpsc::channel();
        (box WinitCompositorProxy {
             sender: sender,
             window_proxy: Some(self.window_proxy.clone()),
         } as Box<compositor_thread::CompositorProxy + Send>,
         box receiver as Box<compositor_thread::CompositorReceiver>)
    }

    fn scale_factor(&self) -> ScaleFactor<f32, ScreenPx, DevicePixel> {
        ScaleFactor::new(self.geometry.get().hidpi_factor)
    }

    fn framebuffer_size(&self) -> TypedSize2D<u32, DevicePixel> {
        let scale_factor = self.geometry.get().hidpi_factor as u32;
        let (width, height) = self.geometry.get().inner_size;
        TypedSize2D::new(scale_factor * width, scale_factor * height)
    }

    fn size(&self) -> TypedSize2D<f32, ScreenPx> {
        let (width, height) = self.geometry.get().inner_size;
        TypedSize2D::new(width as f32, height as f32)
    }

    fn client_window(&self) -> (Size2D<u32>, Point2D<i32>) {
        let (width, height) = self.geometry.get().inner_size;
        let (x, y) = self.geometry.get().position;
        (Size2D::new(width, height), Point2D::new(x as i32, y as i32))
    }

    // Events

    fn set_inner_size(&self, size: Size2D<u32>) {
        self.event_queue.borrow_mut().push(ServoEvent::SetWindowInnerSize(size.width as u32, size.height as u32));
    }

    fn set_position(&self, point: Point2D<i32>) {
        self.event_queue.borrow_mut().push(ServoEvent::SetWindowPosition(point.x, point.y));
    }

    fn set_fullscreen_state(&self, state: bool) {
        self.event_queue.borrow_mut().push(ServoEvent::SetFullScreenState(state))
    }

    fn present(&self) {
        self.event_queue.borrow_mut().push(ServoEvent::Present);
    }

    fn set_page_title(&self, title: Option<String>) {
        self.event_queue.borrow_mut().push(ServoEvent::TitleChanged(title));
    }

    fn set_page_url(&self, url: ServoUrl) {
        self.event_queue.borrow_mut().push(ServoEvent::URLChanged(url));
    }

    fn status(&self, status: Option<String>) {
        self.event_queue.borrow_mut().push(ServoEvent::StatusChanged(status));
    }

    fn load_start(&self, can_go_back: bool, can_go_forward: bool) {
        self.event_queue.borrow_mut().push(ServoEvent::LoadStart(can_go_back, can_go_forward));
    }

    fn load_end(&self, can_go_back: bool, can_go_forward: bool, root: bool) {
        self.event_queue.borrow_mut().push(ServoEvent::LoadEnd(can_go_back, can_go_forward, root));
    }

    fn load_error(&self, _: NetError, url: String) {
        // FIXME: never called by servo
        self.event_queue.borrow_mut().push(ServoEvent::LoadError(url));
    }

    fn head_parsed(&self) {
        self.event_queue.borrow_mut().push(ServoEvent::HeadParsed);
    }

    fn set_cursor(&self, cursor: Cursor) {
        self.event_queue.borrow_mut().push(ServoEvent::CursorChanged(cursor));
    }

    fn set_favicon(&self, url: ServoUrl) {
        self.event_queue.borrow_mut().push(ServoEvent::FaviconChanged(url));
    }

    fn handle_key(&self, ch: Option<char>, key: Key, mods: constellation_msg::KeyModifiers) {
        self.event_queue.borrow_mut().push(ServoEvent::Key(ch, key, mods));
    }
}

struct WinitCompositorProxy {
    sender: std::sync::mpsc::Sender<compositor_thread::Msg>,
    window_proxy: Option<winit::WindowProxy>,
}

impl compositor_thread::CompositorProxy for WinitCompositorProxy {
    fn send(&self, msg: compositor_thread::Msg) {
        if let Err(err) = self.sender.send(msg) {
            println!("Failed to send response ({}).", err);
        }
        if let Some(ref window_proxy) = self.window_proxy {
            window_proxy.wakeup_event_loop()
        }
    }

    fn clone_compositor_proxy
        (&self)
         -> Box<compositor_thread::CompositorProxy + Send> {
        box WinitCompositorProxy {
            sender: self.sender.clone(),
            window_proxy: self.window_proxy.clone(),
        } as Box<compositor_thread::CompositorProxy + Send>
    }
}
