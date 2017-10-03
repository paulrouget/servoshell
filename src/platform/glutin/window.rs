/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use glutin;
use logs::ShellLog;
use platform::View;
use servo::EventLoopWaker;
use state::{BrowserState, ChangeType, DiffKey, WindowState};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use super::GlutinWindow;
use tinyfiledialogs;
use traits::view::ViewMethods;
use traits::window::{WindowCommand, WindowEvent, WindowMethods};

pub struct Window {
    id: glutin::WindowId,
    windows: Rc<RefCell<HashMap<glutin::WindowId, GlutinWindow>>>,
}

impl Window {
    pub fn new(id: glutin::WindowId,
               state: &WindowState,
               windows: Rc<RefCell<HashMap<glutin::WindowId, GlutinWindow>>>)
               -> Window {
        let window = Window { id, windows };
        window.render_title(state);
        window
    }

    fn render_title(&self, state: &WindowState) {
        let text = state
            .tabs
            .alive_browsers()
            .iter()
            .fold("|".to_owned(), |f, b| {
                let title = b.title
                    .as_ref()
                    .and_then(|t| if t.is_empty() { None } else { Some(t) })
                    .map_or("No Title", |t| t.as_str());
                let selected = if !b.background { '>' } else { ' ' };
                let loading = if b.is_loading { '*' } else { ' ' };
                format!("{} {} {:15.15} {}|", f, selected, title, loading)
            });
        let mut windows = self.windows.borrow_mut();
        windows
            .get_mut(&self.id)
            .unwrap()
            .glutin_window
            .set_title(&text);
    }

    fn render_urlbar(&self, state: &BrowserState) {
        if state.urlbar_focused {
            let mut windows = self.windows.borrow_mut();
            let url = format!("{}", state.url.as_ref().map_or("", |t| t.as_str()));
            match tinyfiledialogs::input_box("Search or type URL", "Search or type URL", &url) {
                Some(input) => {
                    let win = windows.get_mut(&self.id).unwrap();
                    win.window_events
                        .push(WindowEvent::DoCommand(WindowCommand::Load(input)));
                }
                None => {}
            }
            windows
                .get_mut(&self.id)
                .unwrap()
                .window_events
                .push(WindowEvent::UrlbarFocusChanged(false));
        }
    }
}

impl WindowMethods for Window {
    fn render(&self, diff: Vec<ChangeType>, state: &WindowState) {

        let idx = state
            .tabs
            .fg_browser_index()
            .expect("no current browser");
        let current_browser_state = state.tabs.ref_fg_browser().expect("no current browser");

        for change in diff {
            use self::DiffKey as K;
            match change {
                ChangeType::Modified(keys) => {
                    match keys.as_slice() {
                        &[K::tabs, K::Index(_), K::Alive, K::background] |
                        &[K::tabs, K::Index(_), K::Alive, K::is_loading] |
                        &[K::tabs, K::Index(_), K::Alive, K::title] => {
                            self.render_title(state);
                        }

                        &[K::status] |
                        &[K::tabs, K::Index(_), K::Alive, K::url] |
                        &[K::tabs, K::Index(_), K::Alive, K::can_go_back] |
                        &[K::tabs, K::Index(_), K::Alive, K::can_go_forward] |
                        &[K::tabs, K::Index(_), K::Alive, K::zoom] |
                        &[K::tabs, K::Index(_), K::Alive, K::user_input] => {
                            // Nothing to do
                        }
                        &[K::tabs, K::Index(i), K::Alive, K::urlbar_focused] if i == idx => {
                            self.render_urlbar(current_browser_state);
                        }
                        _ => println!("Window::render: unexpected Modified keys: {:?}", keys),
                    }
                }
                ChangeType::Added(keys) => {
                    match keys.as_slice() {
                        &[K::tabs, K::Index(_)] => {
                            self.render_title(state);
                        }
                        _ => println!("Window::render: unexpected Added keys: {:?}", keys),
                    }
                }
                ChangeType::Removed(keys) => {
                    match keys.as_slice() {
                        &[K::tabs, K::Index(_), K::Alive] => {
                            self.render_title(state);
                        }
                        _ => println!("Window::render: unexpected Removed keys: {:?}", keys),
                    }
                }
            }
        }
    }

    fn new_view(&self) -> Result<Rc<ViewMethods>, &'static str> {
        Ok(Rc::new(View::new(self.id, self.windows.clone())))
    }

    fn new_event_loop_waker(&self) -> Box<EventLoopWaker> {
        let mut windows = self.windows.borrow_mut();
        windows
            .get_mut(&self.id)
            .unwrap()
            .event_loop_waker
            .clone()
    }

    fn get_events(&self) -> Vec<WindowEvent> {
        let mut windows = self.windows.borrow_mut();
        let win = windows.get_mut(&self.id).unwrap();
        let events = win.window_events.drain(..).collect();
        events
    }

    fn append_logs(&self, _logs: &Vec<ShellLog>) {}
}
