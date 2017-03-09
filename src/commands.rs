#[derive(Clone, Debug)]
pub enum CommandState {
    Enabled,
    Disabled,
}


#[derive(Clone, Debug, Hash, PartialEq, Eq)]
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
    Load(String),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum AppCommand {
    ClearHistory
}
