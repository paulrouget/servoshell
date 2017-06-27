/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub use platform::Window;

#[derive(Clone, Debug)]
pub enum WindowEvent {
    EventLoopAwaken,
    GeometryDidChange,
    DidEnterFullScreen,
    DidExitFullScreen,
    WillClose,
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
    NextTab,
    PrevTab,
    ShowOptions,
    Load(String),
    ToggleOptionShowLogs,
    ToggleOptionLockDomain,
    ToggleOptionFragmentBorders,
    ToggleOptionParallelDisplayListBuidling,
    ToggleOptionShowParallelLayout,
    ToggleOptionConvertMouseToTouch,
    ToggleOptionWebRenderStats,
    ToggleOptionTileBorders,
}
