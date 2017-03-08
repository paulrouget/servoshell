pub use platform::Controls;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ControlEvent {
    Stop,
    Reload,
    GoBack,
    GoForward,
    ZoomIn,
    ZoomOut,
    ZoomToActualSize,
    OpenLocation,
}
