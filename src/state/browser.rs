/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo::BrowserId;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct BrowserState {
    pub id: BrowserId,
    pub is_background: bool,
    pub zoom: f32,
    pub url: Option<String>,
    pub title: Option<String>,
    // FIXME: pub favicon: Option<>,
    pub user_input: Option<String>,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub is_loading: bool,
    pub urlbar_focused: bool,
    // FIXME:
    // creation_timestamp
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct DeadBrowserState {
    pub id: BrowserId,
    // FIXME:
    // close_timestamp,
    // creation_timestamp,
    // load_data: LoadData,
}
