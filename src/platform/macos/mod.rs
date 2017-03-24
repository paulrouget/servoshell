/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod utils;
mod app;
mod window;
mod view;
mod logs;
mod toolbar;
mod bookmarks;

use std::env;
use std::sync::{Once, ONCE_INIT};
use std::path::PathBuf;
use state::AppState;

use cocoa::base::*;
use cocoa::appkit::*;
use std::os::raw::c_void;

pub use self::app::App;
pub use self::window::Window;
pub use self::window::EventLoopRiser;
pub use self::view::View;
pub use self::logs::Logger;

static INIT: Once = ONCE_INIT;

pub fn init() {
    INIT.call_once(|| {
        app::register();
        view::register();
        toolbar::register();
        window::register();
        bookmarks::register();
    });
}

pub fn get_state<'a>() -> &'a mut AppState {
    unsafe {
        let delegate: id = msg_send![NSApp(), delegate];
        let ivar: *mut c_void = *(&*delegate).get_ivar("state");
        &mut *(ivar as *mut AppState)
    }
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
