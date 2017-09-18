/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg_attr(any(feature = "force-glutin", not(target_os = "macos")), allow(dead_code))]

use state::AppState;
use std::path::PathBuf;
use window::WindowMethods;

pub use platform::App;

#[derive(Clone, Debug)]
pub enum AppEvent {
    DidFinishLaunching,
    WillTerminate,
    DidChangeScreenParameters,
    DoCommand(AppCommand),
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppCommand {
    ClearHistory,
    ToggleOptionDarkTheme,
}

pub trait AppMethods {
    fn new<'a>() -> Result<Self, &'a str> where Self: Sized;
    fn new_window<'a>(&self) -> Result<Box<WindowMethods>, &'a str>;
    fn get_init_state() -> AppState;
    fn get_resources_path() -> Option<PathBuf>;
    fn render(&self, state: &AppState);
    fn get_events(&self) -> Vec<AppEvent>;
    fn run<T>(&self, callback: T) where T: Fn();
}
