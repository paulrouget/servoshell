/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use view;
use servo::{BrowserId, ServoCursor};

#[derive(Clone)]
pub struct AppState {
    pub current_window_index: Option<usize>,
    pub windows: Vec<WindowState>,
    pub dark_theme: bool,
    pub cursor: ServoCursor,
}

#[derive(Clone)]
pub struct WindowState {
    pub current_browser_index: Option<usize>,
    pub browsers: Vec<BrowserState>,
    pub sidebar_is_open: bool,
    pub logs_visible: bool,
    pub debug_options: DebugOptions,
    pub status: Option<String>,
    pub urlbar_focused: bool,
    pub options_open: bool,
    pub title: String,
}

#[derive(Clone)]
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
}

#[derive(Clone)]
pub struct DebugOptions {
    pub show_fragment_borders: bool,
    pub parallel_display_list_building: bool,
    pub show_parallel_layout: bool,
    pub convert_mouse_to_touch: bool,
    pub show_tiles_borders: bool,

    // webrender:
    pub wr_profiler: bool,
    pub wr_texture_cache_debug: bool,
    pub wr_render_target_debug: bool,
}
