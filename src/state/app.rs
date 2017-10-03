/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo::ServoCursor;

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct AppState {
    pub current_window_index: Option<usize>,
    pub dark_theme: bool,
    pub cursor: ServoCursor,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            current_window_index: None,
            dark_theme: false,
            cursor: ServoCursor::Default,
        }
    }
}
