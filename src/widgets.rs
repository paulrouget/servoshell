use std::fmt;

#[derive(Clone)]
pub enum WidgetEvent {
    ReloadClicked,
    BackClicked,
    FwdClicked,
}

impl fmt::Debug for WidgetEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WidgetEvent::ReloadClicked => write!(f, "ReloadClicked"),
            WidgetEvent::BackClicked => write!(f, "BackClicked"),
            WidgetEvent::FwdClicked => write!(f, "FwdClicked"),
        }
    }
}

#[cfg(target_os = "macos")]
#[path="platform/cocoa.rs"]
pub mod platform;
