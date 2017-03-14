pub use platform::Window;

#[derive(Clone, Debug)]
pub enum WindowEvent {
    EventLoopRised,
    GeometryDidChange,
    DidEnterFullScreen,
    DidExitFullScreen,
    WillClose,
    DoCommand(WindowCommand),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowCommand {
    Reload,
    Stop,
    NavigateBack,
    NavigateForward,
    OpenLocation,
    OpenInDefaultBrowser,
    ZoomIn,
    ZoomOut,
    ZoomToActualSize,
    ToggleSidebar,
    ShowOptions,
    Load(String),
    ToggleOptionShowLogs,
    ToggleOptionLockDomain,
    ToggleOptionFragmentBorders,
    ToggleOptionParallelDisplayListBuidling,
    ToggleOptionShowParallelLayout,
    ToggleOptionConvertMouseToTouch,
    ToggleOptionCompositorBorders,
    ToggleOptionShowParallelPaint,
    ToggleOptionPaintFlashing,
    ToggleOptionWebRenderStats,
    ToggleOptionMultisampleAntialiasing,
    ToggleOptionTileBorders,
}
