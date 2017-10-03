/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use treediff::{self, Delegate};
use serde::{Deserialize, Serialize};
use serde_json;

pub struct State<T> {
    current_state: T,
    last_state: T,
    has_changed: bool,
}

impl<'t, T> State<T>
    where T: Clone + Deserialize<'t> + Serialize
{
    pub fn new(state: T) -> State<T> {
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

// FIXME: can we generate all of these with macros?
// FIXME: I'm not even sure everything bound to a string

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum DiffKey {
    // Don't keep the string
    Unknown(String),
    Index(usize),
    Alive,
    Dead,
    background,
    dark_theme,
    cursor,
    tabs,
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
                    "Dead" => DiffKey::Dead,
                    "Alive" => DiffKey::Alive,
                    "background" => DiffKey::background,
                    "dark_theme" => DiffKey::dark_theme,
                    "cursor" => DiffKey::cursor,
                    "tabs" => DiffKey::tabs,
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
                    s => DiffKey::Unknown(s.to_owned()),
                }
            }
        }
    }
}

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
        self.changes
            .push(ChangeType::Removed(mk(&self.cursor, Some(k))));
    }
    fn added<'b>(&mut self, k: &'b treediff::value::Key, _v: &'a serde_json::Value) {
        self.changes
            .push(ChangeType::Added(mk(&self.cursor, Some(k))));
    }
    fn modified<'b>(&mut self, _v1: &'a serde_json::Value, _v2: &'a serde_json::Value) {
        self.changes
            .push(ChangeType::Modified(mk(&self.cursor, None)));
    }
    fn unchanged<'b>(&mut self, _v: &'a serde_json::Value) {}
}
