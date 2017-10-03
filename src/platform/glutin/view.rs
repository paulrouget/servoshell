/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use glutin::{self, GlContext};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use super::GlutinWindow;
use traits::view::*;

pub struct View {
    id: glutin::WindowId,
    windows: Rc<RefCell<HashMap<glutin::WindowId, GlutinWindow>>>,
}

impl View {
    pub fn new(id: glutin::WindowId,
               windows: Rc<RefCell<HashMap<glutin::WindowId, GlutinWindow>>>)
               -> View {
        View { id, windows }
    }

    #[cfg(not(target_os = "windows"))]
    fn hidpi_factor(&self) -> f32 {
        let windows = self.windows.borrow();
        let win = windows.get(&self.id).unwrap();
        win.glutin_window.hidpi_factor()
    }

    #[cfg(target_os = "windows")]
    fn hidpi_factor(&self) -> f32 {
        super::utils::windows_hidpi_factor()
    }
}

impl ViewMethods for View {
    fn get_geometry(&self) -> DrawableGeometry {
        let windows = self.windows.borrow();
        let win = windows.get(&self.id).unwrap();
        let (mut width, mut height) = win.glutin_window
            .get_inner_size()
            .expect("Failed to get window inner size.");

        #[cfg(target_os = "windows")]
        let factor = super::utils::windows_hidpi_factor();
        #[cfg(not(target_os = "windows"))]
        let factor = 1.0;

        width /= factor as u32;
        height /= factor as u32;

        DrawableGeometry {
            view_size: (width, height),
            margins: (0, 0, 0, 0),
            position: win.glutin_window
                .get_position()
                .expect("Failed to get window position."),
            hidpi_factor: self.hidpi_factor(),
        }
    }

    fn update_drawable(&self) {
        let windows = self.windows.borrow();
        let win = windows.get(&self.id).unwrap();
        let (w, h) = win.glutin_window
            .get_inner_size()
            .expect("Failed to get window inner size.");
        win.glutin_window.resize(w, h);
    }

    // FIXME: should be controlled by state
    fn enter_fullscreen(&self) {}

    // FIXME: should be controlled by state
    fn exit_fullscreen(&self) {}

    fn set_live_resize_callback(&self, _callback: &FnMut()) {
        // FIXME
    }

    fn gl(&self) -> Rc<gl::Gl> {
        self.windows
            .borrow()
            .get(&self.id)
            .unwrap()
            .gl
            .clone()
    }

    fn get_events(&self) -> Vec<ViewEvent> {
        let mut windows = self.windows.borrow_mut();
        let win = windows.get_mut(&self.id).unwrap();
        let events = win.view_events.drain(..).collect();
        events
    }

    fn swap_buffers(&self) {
        self.windows
            .borrow()
            .get(&self.id)
            .unwrap()
            .glutin_window
            .swap_buffers()
            .unwrap();
    }
}
