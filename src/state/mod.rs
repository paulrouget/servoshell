/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod app;
mod browser;
mod state;
mod tabs;
mod window;

pub use self::state::{DiffKey, ChangeType, State};
pub use self::app::AppState;
pub use self::browser::{BrowserState, DeadBrowserState};
pub use self::window::WindowState;
