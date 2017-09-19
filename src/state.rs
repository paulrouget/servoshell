/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo::{ServoCursor, BrowserId};

#[derive(Clone)]
pub struct AppState {
    pub current_window_index: Option<usize>,
    pub windows: Vec<WindowState>,
    pub dark_theme: bool,
    pub cursor: ServoCursor,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            current_window_index: None,
            windows: Vec::new(),
            dark_theme: false,
            cursor: ServoCursor::Default,
        }
    }
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

impl WindowState {
    pub fn new() -> WindowState {
        WindowState {
            current_browser_index: None,
            browsers: Vec::new(),
            sidebar_is_open: false,
            logs_visible: false,
            status: None,
            urlbar_focused: false,
            options_open: false,
            title: "ServoShell".to_owned(),
            debug_options: DebugOptions {
                show_fragment_borders: false,
                parallel_display_list_building: false,
                show_parallel_layout: false,
                convert_mouse_to_touch: false,
                show_tiles_borders: false,
                wr_profiler: false,
                wr_texture_cache_debug: false,
                wr_render_target_debug: false,
            },
        }
    }
}

#[derive(Clone)]
pub struct BrowserState {
    pub id: BrowserId,
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
