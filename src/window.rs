pub use platform::Window;
use commands::WindowCommand;

#[derive(Clone, Debug)]
pub enum WindowEvent {
    EventLoopRised,
    GeometryDidChange,
    DidEnterFullScreen,
    DidExitFullScreen,
    WillClose,
    DoCommand(WindowCommand),
}
