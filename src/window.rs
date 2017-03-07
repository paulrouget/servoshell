pub use platform::Window;

#[derive(Clone, Debug)]
pub enum WindowEvent {
    EventLoopRised,
    GeometryDidChange,
    DidEnterFullScreen,
    DidExitFullScreen,
    WillClose,
}

