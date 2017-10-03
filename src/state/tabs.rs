/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo::BrowserId;
use super::{BrowserState, DeadBrowserState};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum TabState {
    Alive(BrowserState),
    Dead(DeadBrowserState),
}

impl TabState {
    fn is_alive(&self) -> bool {
        match *self {
            TabState::Alive(_) => true,
            TabState::Dead(_) => false,
        }
    }
    fn is_fg(&self) -> bool {
        self.ref_browser()
            .map(|b| !b.background)
            .unwrap_or(false)
    }
    fn is_bg(&self) -> bool {
        self.ref_browser()
            .map(|b| b.background)
            .unwrap_or(false)
    }
    fn ref_browser(&self) -> Result<&BrowserState, &'static str> {
        match *self {
            TabState::Alive(ref x) => Ok(x),
            TabState::Dead(_) => Err("Dead browser"),
        }
    }
    fn mut_browser(&mut self) -> Result<&mut BrowserState, &'static str> {
        match *self {
            TabState::Alive(ref mut x) => Ok(x),
            TabState::Dead(_) => Err("Dead browser"),
        }
    }
    fn kill(&mut self) -> Result<(), &'static str> {
        if !self.is_alive() {
            return Err("Already dead");
        }
        let id = self.ref_browser().unwrap().id;
        let tab = TabState::Dead(DeadBrowserState { id });
        *self = tab;
        Ok(())
    }
    fn foreground(&mut self) -> Result<(), &'static str> {
        match *self {
            TabState::Alive(ref mut browser) if browser.background => {
                browser.background = false;
                Ok(())
            }
            TabState::Alive(_) => Err("Already foreground"),
            TabState::Dead(_) => Err("Dead browser"),
        }
    }
    fn background(&mut self) -> Result<(), &'static str> {
        match *self {
            TabState::Alive(ref mut browser) if !browser.background => {
                browser.background = true;
                Ok(())
            }
            TabState::Alive(_) => Err("Already background"),
            TabState::Dead(_) => Err("Dead browser"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TabsState(Vec<TabState>);

#[allow(dead_code)]
impl TabsState {
    pub fn new() -> TabsState {
        TabsState(Vec::new())
    }

    pub fn has_more_than_one(&self) -> bool {
        self.0.iter().filter(|tab| tab.is_alive()).count() > 1
    }

    pub fn kill_fg(&mut self) -> Result<BrowserId, &'static str> {
        let fg_idx = self.0
            .iter()
            .position(TabState::is_fg)
            .ok_or("No foreground tab")?;
        if self.can_select_next()? {
            self.select_next()?;
        } else if self.can_select_prev()? {
            self.select_prev()?;
        } else {
            return Err("No background tab to select");
        }
        let id = self.0[fg_idx].ref_browser()?.id;
        self.0[fg_idx].kill()?;
        Ok(id)
    }

    pub fn can_select_next(&self) -> Result<bool, &'static str> {
        let fg_idx = self.0
            .iter()
            .position(TabState::is_fg)
            .ok_or("No foreground tab")?;
        Ok(self.0.iter().skip(fg_idx + 1).any(TabState::is_bg))
    }

    pub fn select_next(&mut self) -> Result<(), &'static str> {
        let fg_idx = self.0
            .iter()
            .position(|tab| tab.is_fg())
            .ok_or("No foreground tab")?;
        let next_idx = self.0
            .iter()
            .enumerate()
            .skip(fg_idx + 1)
            .find(|&(_, tab)| tab.is_bg())
            .map(|(idx, _)| idx)
            .ok_or("No tab to select")?;
        self.0[fg_idx].background()?;
        self.0[next_idx].foreground()?;
        Ok(())
    }

    pub fn can_select_prev(&self) -> Result<bool, &'static str> {
        let fg_idx = self.0
            .iter()
            .position(TabState::is_fg)
            .ok_or("No foreground tab")?;
        Ok(self.0
               .iter()
               .rev()
               .skip(self.0.len() - fg_idx)
               .any(TabState::is_bg))
    }

    pub fn select_prev(&mut self) -> Result<(), &'static str> {
        let fg_idx = self.0
            .iter()
            .position(TabState::is_fg)
            .ok_or("No foreground tab")?;
        let prev_idx = self.0
            .iter()
            .enumerate()
            .rev()
            .skip(self.0.len() - fg_idx)
            .find(|&(_, tab)| tab.is_bg())
            .map(|(idx, _)| idx)
            .ok_or("No tab to select")?;
        self.0[fg_idx].background()?;
        self.0[prev_idx].foreground()?;
        Ok(())
    }

    pub fn select_first(&mut self) -> Result<(), &'static str> {
        let fg_idx = self.0
            .iter()
            .position(TabState::is_fg)
            .ok_or("No foreground tab")?;
        let first_idx = self.0
            .iter()
            .position(TabState::is_bg)
            .ok_or("No tab to select")?;
        self.0[fg_idx].background()?;
        self.0[first_idx].foreground()?;
        Ok(())
    }

    pub fn select_last(&mut self) -> Result<(), &'static str> {
        let fg_idx = self.0
            .iter()
            .position(TabState::is_fg)
            .ok_or("No foreground tab")?;
        let last_idx = self.0
            .iter()
            .enumerate()
            .rev()
            .find(|&(_, tab)| tab.is_bg())
            .map(&|(idx, _)| idx)
            .ok_or("No tab to select")?;
        self.0[fg_idx].background()?;
        self.0[last_idx].foreground()?;
        Ok(())
    }

    pub fn can_select_nth(&self, index: usize) -> bool {
        self.0
            .iter()
            .filter(|tab| tab.is_alive())
            .nth(index)
            .is_some()
    }

    pub fn select_nth(&mut self, index: usize) -> Result<(), &'static str> {
        let fg_idx = self.0
            .iter()
            .position(TabState::is_fg)
            .ok_or("No foreground tab")?;
        let nth_idx = self.0
            .iter()
            .enumerate()
            .filter(|&(_, tab)| tab.is_alive())
            .nth(index)
            .map(|(index, _)| index)
            .ok_or("No tab to select")?;
        self.0[fg_idx].background()?;
        self.0[nth_idx].foreground()?;
        Ok(())
    }

    pub fn append_new(&mut self, mut browser: BrowserState) -> Result<(), &'static str> {
        if self.0.len() == 0 {
            browser.background = false;
            self.0.push(TabState::Alive(browser));
            Ok(())
        } else if !browser.background {
            browser.background = true;
            self.0.push(TabState::Alive(browser));
            self.select_last()
        } else {
            self.0.push(TabState::Alive(browser));
            Ok(())
        }
    }

    pub fn find_browser(&mut self, id: &BrowserId) -> Option<&mut BrowserState> {
        self.0
            .iter_mut()
            .filter_map(|tab| tab.mut_browser().ok())
            .find(|b| b.id == *id)
    }

    pub fn find_browser_at(&self, idx: usize) -> Option<&BrowserState> {
        self.0
            .iter()
            .nth(idx)
            .and_then(|tab| tab.ref_browser().ok())
    }

    pub fn index_to_alive_index(&self, idx: usize) -> Option<usize> {
        // We need to also check tab == found_tab as we don't want
        // to discard a dead tab. The user might want to know the
        // position of what used to be an alive tab
        self.0
            .iter()
            .nth(idx)
            .and_then(|found_tab| {
                          self.0
                              .iter()
                              .filter(|&tab| tab.is_alive() || tab == found_tab)
                              .position(|tab| tab == found_tab)
                      })
    }

    pub fn ref_fg_browser(&self) -> Result<&BrowserState, &'static str> {
        self.0
            .iter()
            .find(|tab| tab.is_fg())
            .ok_or("No foreground tab")
            .and_then(|tab| tab.ref_browser())
    }

    pub fn mut_fg_browser(&mut self) -> Result<&mut BrowserState, &'static str> {
        self.0
            .iter_mut()
            .find(|tab| tab.is_fg())
            .ok_or("No foreground tab")
            .and_then(|tab| tab.mut_browser())
    }

    pub fn fg_browser_index(&self) -> Result<usize, &'static str> {
        self.0
            .iter()
            .position(|tab| tab.is_fg())
            .ok_or("No foreground tab")
    }

    pub fn alive_browsers<'a>(&self) -> Vec<&BrowserState> {
        self.0
            .iter()
            .filter_map(|tab| tab.ref_browser().ok())
            .collect()
    }
}
