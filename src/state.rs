/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use treediff::{self, Delegate};
use servo::{BrowserId, ServoCursor};

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
    pub urlbar_focused: bool,
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

    pub fn diff(&self, to: &AppState) {
        use serde_json;

        let from = serde_json::to_value(self).unwrap();
        let to = serde_json::to_value(to).unwrap();

        let mut recorder = DiffRecorder::new();
        treediff::diff(&from, &to, &mut recorder);
        recorder.changes

        let _: Vec<()> = changes.iter().map(|change| {
            match *change {
                ChangeType::Modified(ref keys, _, value) => {
                    match keys[0] {
                        DiffKey::cursor => {
                            println!("mop mop");
                        },
                        _ => {
                            println!("stuff");
                        },

                    }
                    println!("modified: {:?} -> {:?}", keys, value);
                    // if keys[0] == CURSOR_KEY {
                    //     let cursor: ServoCursor = serde_json::from_value(value.clone()).unwrap();
                    //     self.update_cursor(cursor);
                    // } else if keys[0] == WINDOW_KEY {
                    //     println!("window");
                    // } else {
                    //     println!("else");
                    // }
                },
                _ => {
                    println!("That should never happen");
                }
            }
        }).collect();
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

/* --------------------------------------------- */

#[derive(Debug, PartialEq)]
pub enum ChangeType<'a, V: 'a> {
    Removed(Vec<DiffKey>, &'a V),
    Added(Vec<DiffKey>, &'a V),
    Modified(Vec<DiffKey>, &'a V, &'a V),
}

#[derive(Debug, PartialEq)]
struct DiffRecorder<'a, V: 'a> {
    cursor: Vec<DiffKey>,
    pub changes: Vec<ChangeType<'a, V>>,
}

impl<'a, V> DiffRecorder<'a, V> {
    fn new() -> DiffRecorder<'a, V> {
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

impl<'a, V> Delegate<'a, treediff::value::Key, V> for DiffRecorder<'a, V> {
    fn push<'b>(&mut self, k: &'b treediff::value::Key) {
        self.cursor.push(DiffKey::from_key(k))
    }
    fn pop(&mut self) {
        self.cursor.pop();
    }
    fn removed<'b>(&mut self, k: &'b treediff::value::Key, v: &'a V) {
        self.changes.push(ChangeType::Removed(mk(&self.cursor, Some(k)), v));
    }
    fn added<'b>(&mut self, k: &'b treediff::value::Key, v: &'a V) {
        self.changes.push(ChangeType::Added(mk(&self.cursor, Some(k)), v));
    }
    fn modified<'b>(&mut self, v1: &'a V, v2: &'a V) {
        self.changes.push(ChangeType::Modified(mk(&self.cursor, None), v1, v2));
    }
    fn unchanged<'b>(&mut self, _v: &'a V) {
    }
}
