/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg_attr(any(feature = "force-glutin", not(target_os = "macos")), allow(dead_code))]

use state::{ChangeType, WindowState};
use traits::view::ViewMethods;
use servo::EventLoopWaker;
use std::rc::Rc;
use logs::ShellLog;

#[derive(Clone, Debug)]
pub enum WindowEvent {
    EventLoopAwaken,
    GeometryDidChange,
    DidEnterFullScreen,
    DidExitFullScreen,
    WillClose,
    OptionsClosed,
    UrlbarFocusChanged(bool),
    DoCommand(WindowCommand),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowCommand {
    Reload,
    Stop,
    NavigateBack,
    NavigateForward,
    OpenLocation,
    OpenInDefaultBrowser,
    ZoomIn,
    ZoomOut,
    ZoomToActualSize,
    ToggleSidebar,
    NewTab,
    CloseTab,
    NextTab,
    PrevTab,
    SelectTab(usize),
    ShowOptions,
    Load(String),
    ToggleOptionShowLogs,
    ToggleOptionFragmentBorders,
    ToggleOptionParallelDisplayListBuidling,
    ToggleOptionShowParallelLayout,
    ToggleOptionConvertMouseToTouch,
    ToggleOptionTileBorders,
    ToggleOptionWRProfiler,
    ToggleOptionWRTextureCacheDebug,
    ToggleOptionWRTargetDebug,
}

pub trait WindowMethods {
    fn render(&self, diff: Vec<ChangeType>, state: &WindowState);
    fn new_view(&self) -> Result<Rc<ViewMethods>, &'static str>;
    fn new_event_loop_waker(&self) -> Box<EventLoopWaker>;
    fn get_events(&self) -> Vec<WindowEvent>;
    fn append_logs(&self, logs: &Vec<ShellLog>);
}
