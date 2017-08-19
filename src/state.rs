/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use view;
use servo::BrowserId;

pub struct AppState {
    pub current_window_index: Option<usize>,
    pub window_states: Vec<WindowState>,
    pub dark_theme: bool,
}

pub struct WindowState {
    pub current_browser_index: Option<usize>,
    pub browser_states: Vec<BrowserState>,
    pub sidebar_is_open: bool,
    pub logs_visible: bool,
}

pub struct BrowserState {
    pub id: BrowserId,
    pub last_mouse_point: (i32, i32),
    pub last_mouse_down_point: (i32, i32),
    pub last_mouse_down_button: Option<view::MouseButton>,
    pub zoom: f32,
    pub url: Option<String>,
    pub title: Option<String>,
    // FIXME: pub favicon: Option<>,
    pub user_input: Option<String>,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub is_loading: bool,
    pub show_fragment_borders: bool,
    pub parallel_display_list_building: bool,
    pub show_parallel_layout: bool,
    pub convert_mouse_to_touch: bool,
    pub show_webrender_stats: bool,
    pub show_tiles_borders: bool,
}
