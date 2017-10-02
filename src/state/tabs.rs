/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo::BrowserId;
use super::{BrowserState, DeadBrowserState};

// FIXME: don't use an option. We want DeadBrowserState instead of None.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TabState(Option<BrowserState>);

impl TabState {
    fn is_alive(&self) -> bool {
        !self.0.is_none()
    }
    fn is_fg(&self) -> bool {
        self.0.map(|b| !b.background).unwrap_or(false)
    }
    fn is_bg(&self) -> bool {
        self.0.map(|b| !b.background).unwrap_or(false)
    }
    fn ref_browser(&self) -> Result<&BrowserState, &'static str> {
        self.0.as_ref().ok_or("Dead browser")
    }
    fn mut_browser(&mut self) -> Result<&mut BrowserState, &'static str> {
        self.0.as_mut().ok_or("Dead browser")
    }
    fn kill(&mut self) -> Result<(), &'static str> {
        match self.0 {
            None => Err("Already dead"),
            Some(_) => {
                self.0 = None;
                Ok(())
            }
        }
    }
    fn foreground(&mut self) -> Result<(), &'static str> {
        match self.0 {
            None => Err("Dead tab"),
            Some(browser) if browser.background => {
                    browser.background = false;
                    Ok(())
            },
            Some(_) => {
                Err("Already foreground")
            }
        }
    }
    fn background(&mut self) -> Result<(), &'static str> {
        match self.0 {
            None => Err("Dead tab"),
            Some(browser) if !browser.background => {
                    browser.background = true;
                    Ok(())
            },
            Some(_) => {
                Err("Already background")
            }
        }
    }
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct TabsState(Vec<TabState>);

impl TabsState {
    pub fn new() -> TabsState {
        TabsState(Vec::new())
    }
    pub fn has_more_than_one_tab(&self) -> bool {
        self.0.iter().filter(|tab| tab.is_alive()).count() > 1
    }
    pub fn kill_fg_tab(&mut self) -> Result<BrowserId, &'static str> {
        let fg_idx = self.0.iter().position(TabState::is_fg).ok_or("No foreground tab")?;
        if self.can_select_next_tab()? {
            self.select_next_tab()?;
        } else if self.can_select_prev_tab()? {
            self.select_prev_tab()?;
        } else {
            return Err("No background tab to select");
        }
        let id = self.0[fg_idx].ref_browser()?.id;
        self.0[fg_idx].kill()?;
        Ok(id)
    }
    pub fn can_select_next_tab(&self) -> Result<bool,&'static str> {
        let fg_idx = self.0.iter().position(TabState::is_fg).ok_or("No foreground tab")?;
        Ok(self.0.iter().skip(fg_idx + 1).any(TabState::is_bg))
    }
    pub fn select_next_tab(&mut self) -> Result<(), &'static str> {
        let fg_idx = self.0.iter()
            .position(TabState::is_fg)
            .ok_or("No foreground tab")?;
        let next_idx = self.0.iter()
            .skip(fg_idx + 1)
            .position(TabState::is_bg)
            .ok_or("No tab to select")?;
        self.0[fg_idx].background()?;
        self.0[next_idx].foreground()?;
        Ok(())
    }
    pub fn can_select_prev_tab(&self) -> Result<bool, &'static str> {
        let fg_idx = self.0.iter().position(TabState::is_fg).ok_or("No foreground tab")?;
        Ok(self.0.iter().rev().skip(self.0.len() - fg_idx).any(TabState::is_bg))
    }
    pub fn select_prev_tab(&mut self) -> Result<(), &'static str> {
        let fg_idx = self.0.iter()
            .position(TabState::is_fg)
            .ok_or("No foreground tab")?;
        let prev_idx = self.0.iter()
            .rev().skip(self.0.len() - fg_idx)
            .position(TabState::is_bg)
            .ok_or("No tab to select")?;
        self.0[fg_idx].background()?;
        self.0[prev_idx].foreground()?;
        Ok(())
    }
    pub fn select_first_tab(&mut self) -> Result<(), &'static str> {
        let fg_idx = self.0.iter()
            .position(TabState::is_fg)
            .ok_or("No foreground tab")?;
        let first_idx = self.0.iter()
            .position(TabState::is_bg)
            .ok_or("No tab to select")?;
        self.0[fg_idx].background()?;
        self.0[first_idx].foreground()?;
        Ok(())
    }
    pub fn select_last_tab(&mut self) -> Result<(), &'static str> {
        let fg_idx = self.0.iter()
            .position(TabState::is_fg)
            .ok_or("No foreground tab")?;
        let last_idx = self.0.iter().rev()
            .position(TabState::is_bg)
            .ok_or("No tab to select")?;
        self.0[fg_idx].background()?;
        self.0[last_idx].foreground()?;
        Ok(())
    }

    pub fn can_select_nth_tab(&self, index: usize) -> bool {
        self.0.iter().filter(|tab| tab.is_alive()).nth(index).is_some()
    }
    pub fn select_nth_tab(&mut self, index: usize) -> Result<(), &'static str> {
        let fg_idx = self.0.iter()
            .position(TabState::is_fg)
            .ok_or("No foreground tab")?;
        let nth_idx = self.0.iter()
            .enumerate()
            .filter(|&(_, tab)| tab.is_alive())
            .nth(index)
            .map(|(index, _)| index)
            .ok_or("No tab to select")?;
        self.0[fg_idx].background()?;
        self.0[nth_idx].foreground()?;
        Ok(())
    }

    pub fn add_new_foreground_tab_at_the_end(&mut self, mut browser: BrowserState) -> Result<(), &'static str> {
        browser.background = false;
        self.0.push(TabState(Some(browser)));
        self.select_last_tab()
    }

    pub fn find_browser(&mut self, id: &BrowserId) -> Option<&mut BrowserState> {
        self.0.iter_mut().filter_map(|tab| tab.mut_browser().ok()).find(|b| b.id == *id)
    }

    // pub fn get_fg_index(&self) -> Result<usize, &'static str> {
    //     self.0.iter().filter_map
    // }

    // pub fn alive_index(&self, usize) -> Result<usize, &'static str> {
    //     self.0.iter().filter(TabState::is_alive).position(|tab| tab.is_fg()).ok_or("no current browser")
    // }
}
