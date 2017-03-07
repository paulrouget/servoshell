mod utils;
mod app;
mod controls;
mod window;
mod toolbar;
mod view;

use std::sync::{Once, ONCE_INIT};

static INIT: Once = ONCE_INIT;

pub fn init() {
    INIT.call_once(|| {
        app::register();
        toolbar::register();
        view::register();
        window::register();
    });
}

pub use self::app::App;
pub use self::window::Window;
pub use self::window::EventLoopRiser;
pub use self::view::View;
pub use self::controls::Controls;
