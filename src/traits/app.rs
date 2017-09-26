/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg_attr(any(feature = "force-glutin", not(target_os = "macos")), allow(dead_code))]

use state::{AppState, ChangeType, WindowState};
use std::path::PathBuf;
use traits::window::WindowMethods;

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
    fn new<'a>(state: &AppState) -> Result<Self, &'a str> where Self: Sized;
    fn new_window<'a>(&self, state: &WindowState) -> Result<Box<WindowMethods>, &'a str>;
    fn get_resources_path() -> Option<PathBuf>;
    fn render(&self, diff: Vec<ChangeType>, state: &AppState);
    fn get_events(&self) -> Vec<AppEvent>;
    fn run<T>(&self, callback: T) where T: FnMut();
}
