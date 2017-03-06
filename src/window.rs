pub use platform::Window;

#[derive(Clone, Debug)]
pub enum WindowEvent {
    EventLoopRised,
    GeometryDidChange,
    WillEnterFullScreen,
    DidEnterFullScreen,
    WillExitFullScreen,
    DidExitFullScreen,
    WillClose,

    ReloadClicked,
    StopClicked,
    HistoryBackClicked,
    HistoryForwardClicked,
    ZoomInClicked,
    ZoomOutClicked,
    ResetZoomClicked,
}

