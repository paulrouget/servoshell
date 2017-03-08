mod utils;
mod app;
mod controls;
mod window;
mod toolbar;
mod view;

use std::env;
use std::sync::{Once, ONCE_INIT};
use std::path::PathBuf;

static INIT: Once = ONCE_INIT;

pub fn init() {
    INIT.call_once(|| {
        app::register();
        controls::register();
        toolbar::register();
        view::register();
        window::register();
    });
}

// Where to find servo_resources/ and nibs/
pub fn get_resources_path() -> Option<PathBuf> {
    // Try current directory. Used for example with "cargo run"
    let p = env::current_dir().unwrap();

    if p.join("servo_resources/").exists() {
        return Some(p)
    }

    // Maybe we run from an app bundle
    let p = env::current_exe().unwrap();
    let p = p.parent().unwrap();
    let p = p.parent().unwrap().join("Resources");

    if p.join("servo_resources/").exists() {
        return Some(p)
    }

    None
}

pub use self::app::App;
pub use self::window::Window;
pub use self::window::EventLoopRiser;
pub use self::view::View;
pub use self::controls::Controls;
