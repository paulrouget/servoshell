/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg_attr(any(feature = "force-glutin", not(target_os = "macos")), allow(dead_code))]

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
