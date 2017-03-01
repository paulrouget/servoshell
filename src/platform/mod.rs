pub use self::platform::*;

#[cfg(target_os = "macos")]
#[path="macos/mod.rs"]
mod platform;
