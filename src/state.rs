/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use treediff::{self, Delegate};
use servo::{BrowserId, ServoCursor};
use serde::{Deserialize, Serialize};
use serde_json;

pub struct State<T> {
    current_state: T,
    last_state: T,
    has_changed: bool,
}

impl<'t, T> State<T> where T: Clone + Deserialize<'t> + Serialize {
    pub fn new(state: T) -> State<T>  {
        State {
            last_state: state.clone(),
            current_state: state,
            has_changed: false,
        }
    }

    pub fn get(&self) -> &T {
        &self.current_state
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.has_changed = true;
        &mut self.current_state
    }

    pub fn snapshot(&mut self) {
        self.has_changed = false;
        self.last_state = self.current_state.clone();
    }

    pub fn diff<'a>(&self) -> Vec<ChangeType> {
        if self.has_changed() {
            let from = serde_json::to_value(&self.last_state).unwrap();
            let to = serde_json::to_value(&self.current_state).unwrap();
            let mut recorder = DiffRecorder::new();
            treediff::diff(&from, &to, &mut recorder);
            recorder.changes
        } else {
            vec![]
        }
    }

    pub fn has_changed(&self) -> bool {
        self.has_changed
    }
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct AppState {
    pub current_window_index: Option<usize>,
    pub dark_theme: bool,
    pub cursor: ServoCursor,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct WindowState {
    pub current_browser_index: Option<usize>,
    pub browsers: Vec<BrowserState>,
    pub sidebar_is_open: bool,
    pub logs_visible: bool,
    pub debug_options: DebugOptions,
    pub status: Option<String>,
    pub options_open: bool,
    pub title: String,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
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
    pub urlbar_focused: bool,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
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

// FIXME: can we generate all of these with macros?

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum DiffKey {
    Unknown,
    Index(usize),
    current_window_index,
    dark_theme,
    cursor,
    current_browser_index,
    browsers,
    sidebar_is_open,
    logs_visible,
    debug_options,
    status,
    urlbar_focused,
    options_open,
    title,
    id,
    zoom,
    url,
    user_input,
    can_go_back,
    can_go_forward,
    is_loading,
    show_fragment_borders,
    parallel_display_list_building,
    show_parallel_layout,
    convert_mouse_to_touch,
    show_tiles_borders,
    wr_profiler,
    wr_texture_cache_debug,
    wr_render_target_debug,
}

impl DiffKey {
    fn from_key(key: &treediff::value::Key) -> DiffKey {
        use treediff::value::Key::{Index, String};
        match *key {
            Index(idx) => DiffKey::Index(idx),
            String(ref name) => {
                match name.as_ref() {
                    "current_window_index" => DiffKey::current_window_index,
                    "dark_theme" => DiffKey::dark_theme,
                    "cursor" => DiffKey::cursor,
                    "current_browser_index" => DiffKey::current_browser_index,
                    "browsers" => DiffKey::browsers,
                    "sidebar_is_open" => DiffKey::sidebar_is_open,
                    "logs_visible" => DiffKey::logs_visible,
                    "debug_options" => DiffKey::debug_options,
                    "status" => DiffKey::status,
                    "urlbar_focused" => DiffKey::urlbar_focused,
                    "options_open" => DiffKey::options_open,
                    "id" => DiffKey::id,
                    "zoom" => DiffKey::zoom,
                    "url" => DiffKey::url,
                    "title" => DiffKey::title,
                    "user_input" => DiffKey::user_input,
                    "can_go_back" => DiffKey::can_go_back,
                    "can_go_forward" => DiffKey::can_go_forward,
                    "is_loading" => DiffKey::is_loading,
                    "show_fragment_borders" => DiffKey::show_fragment_borders,
                    "parallel_display_list_building" => DiffKey::parallel_display_list_building,
                    "show_parallel_layout" => DiffKey::show_parallel_layout,
                    "convert_mouse_to_touch" => DiffKey::convert_mouse_to_touch,
                    "show_tiles_borders" => DiffKey::show_tiles_borders,
                    "wr_profiler" => DiffKey::wr_profiler,
                    "wr_texture_cache_debug" => DiffKey::wr_texture_cache_debug,
                    "wr_render_target_debug" => DiffKey::wr_render_target_debug,
                    _ => DiffKey::Unknown,
                }
            }
        }
    }
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

impl WindowState {
    pub fn new() -> WindowState {
        WindowState {
            current_browser_index: None,
            browsers: Vec::new(),
            sidebar_is_open: false,
            logs_visible: false,
            status: None,
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

/* --------------------------------------------- */

#[derive(Debug, PartialEq)]
pub enum ChangeType {
    Removed(Vec<DiffKey>),
    Added(Vec<DiffKey>),
    Modified(Vec<DiffKey>),
}

#[derive(Debug, PartialEq)]
struct DiffRecorder {
    cursor: Vec<DiffKey>,
    pub changes: Vec<ChangeType>,
}

impl DiffRecorder {
    fn new() -> DiffRecorder {
        DiffRecorder {
            cursor: Vec::new(),
            changes: Vec::new(),
        }
    }
}

fn mk<'b>(c: &Vec<DiffKey>, k: Option<&'b treediff::value::Key>) -> Vec<DiffKey> {
    let mut c = c.clone();
    match k {
        Some(k) => {
            c.push(DiffKey::from_key(k));
            c
        }
        None => c,
    }
}

impl<'a> Delegate<'a, treediff::value::Key, serde_json::Value> for DiffRecorder {
    fn push<'b>(&mut self, k: &'b treediff::value::Key) {
        self.cursor.push(DiffKey::from_key(k))
    }
    fn pop(&mut self) {
        self.cursor.pop();
    }
    fn removed<'b>(&mut self, k: &'b treediff::value::Key, _v: &'a serde_json::Value) {
        self.changes.push(ChangeType::Removed(mk(&self.cursor, Some(k))));
    }
    fn added<'b>(&mut self, k: &'b treediff::value::Key, _v: &'a serde_json::Value) {
        self.changes.push(ChangeType::Added(mk(&self.cursor, Some(k))));
    }
    fn modified<'b>(&mut self, _v1: &'a serde_json::Value, _v2: &'a serde_json::Value) {
        self.changes.push(ChangeType::Modified(mk(&self.cursor, None)));
    }
    fn unchanged<'b>(&mut self, _v: &'a serde_json::Value) {
    }
}
