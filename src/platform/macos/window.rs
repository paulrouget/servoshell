/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cocoa::appkit::*;
use cocoa::foundation::*;
use cocoa::base::*;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use std::f64;
use std::ffi::CStr;
use std::os::raw::c_void;
use super::utils::{self, get_mut_state, get_state};
use window::{WindowEvent, WindowCommand};
use view::View;
use libc;
use servo::EventLoopWaker;
use state::{DebugOptions, WindowState};
use super::logs::ShellLog;

#[link(name = "MMTabBarView", kind = "framework")]
#[link_args = "-rpath target/MMTabBarView/Release/ -rpath @executable_path/../Frameworks/"]
extern { }

pub fn register() {

    /* NSWindow subclass */ {

        let superclass = Class::get("NSWindow").unwrap();
        let mut class = ClassDecl::new("NSShellWindow", superclass).unwrap();
        class.add_ivar::<*mut c_void>("event_queue");

        extern fn toggle_tabbar(this: &Object, _sel: Sel, sender: id) {
            unsafe {
                msg_send![super(this, Class::get("NSWindow").unwrap()), toggleTabBar:sender];
            }
            utils::get_event_queue(this).push(WindowEvent::GeometryDidChange);
        }

        extern fn toggle_toolbar(this: &Object, _sel: Sel, sender: id) {
            unsafe {
                msg_send![super(this, Class::get("NSWindow").unwrap()), toggleToolbarShown:sender];
            }
            utils::get_event_queue(this).push(WindowEvent::GeometryDidChange);
        }

        extern fn event_loop_awaken(this: &Object, _sel: Sel) {
            // FIXME: wut?
            utils::get_event_queue(this).push(WindowEvent::EventLoopAwaken);
        }

        extern fn awake_from_nib(this: &mut Object, _sel: Sel) {
            let event_queue: Vec<WindowEvent> = Vec::new();
            // FIXME: is that the best way to create a raw pointer?
            let event_queue_ptr = Box::into_raw(Box::new(event_queue));
            unsafe {
                this.set_ivar("event_queue", event_queue_ptr as *mut c_void);
            }
        }

        unsafe {
            class.add_method(sel!(toggleTabBar:), toggle_tabbar as extern fn(&Object, Sel, id));
            class.add_method(sel!(toggleToolbarShown:), toggle_toolbar as extern fn(&Object, Sel, id));
            class.add_method(sel!(eventLoopAwaken), event_loop_awaken as extern fn(&Object, Sel));
            class.add_method(sel!(awakeFromNib), awake_from_nib as extern fn(&mut Object, Sel));
        }

        class.register();
    }

    /* NSWindowDelegate */ {

        let superclass = Class::get("NSObject").unwrap();
        let mut class = ClassDecl::new("NSShellWindowDelegate", superclass).unwrap();
        class.add_ivar::<*mut c_void>("event_queue");
        class.add_ivar::<bool>("rendering");

        // FIXME: Don't use strings. And maybe use a map to avoid the duplicate code with add_method.
        extern fn record_notification(this: &Object, _sel: Sel, notification: id) {
            let event = unsafe {
                let name: id = msg_send![notification, name];
                if NSString::isEqualToString(name, "NSWindowDidEnterFullScreenNotification") {
                    Some(WindowEvent::DidEnterFullScreen)
                } else if NSString::isEqualToString(name, "NSWindowDidExitFullScreenNotification") {
                    Some(WindowEvent::DidExitFullScreen)
                } else if NSString::isEqualToString(name, "NSWindowWillCloseNotification") {
                    Some(WindowEvent::WillClose)
                } else {
                    None
                }
            };
            utils::get_event_queue(this).push(event.unwrap());
        }

        extern fn record_command(this: &Object, _sel: Sel, item: id) {
            let action: Sel = unsafe {msg_send![item, action]};
            let cmd = if action == sel!(shellNavigate:) {
                let idx: NSInteger = unsafe { msg_send![item, selectedSegment] };
                if idx == 0 {
                    WindowCommand::NavigateBack
                } else {
                    WindowCommand::NavigateForward
                }
            } else if action == sel!(shellZoom:) {
                let idx: NSInteger = unsafe { msg_send![item, selectedSegment] };
                if idx == 0 {
                    WindowCommand::ZoomOut
                } else  if idx == 1 {
                    WindowCommand::ZoomToActualSize
                } else {
                    WindowCommand::ZoomIn
                }
            } else if action == sel!(shellReloadStop:) {
                let idx = get_state().windows[0].current_browser_index.unwrap();
                if get_state().windows[0].browsers[idx].is_loading {
                    WindowCommand::Stop
                } else {
                    WindowCommand::Reload
                }
            } else if action == sel!(shellStop:) { WindowCommand::Stop }
            else if action == sel!(shellReload:) { WindowCommand::Reload }
            else if action == sel!(shellOpenLocation:) { WindowCommand::OpenLocation }
            else if action == sel!(shellNewTab:) { WindowCommand::NewTab }
            else if action == sel!(shellCloseTab:) { WindowCommand::CloseTab}
            else if action == sel!(shellNextTab:) { WindowCommand::NextTab}
            else if action == sel!(shellPrevTab:) { WindowCommand::PrevTab}
            else if action == sel!(shellZoomIn:) { WindowCommand::ZoomIn }
            else if action == sel!(shellZoomOut:) { WindowCommand::ZoomOut }
            else if action == sel!(shellZoomToActualSize:) { WindowCommand::ZoomToActualSize }
            else if action == sel!(shellNavigateBack:) { WindowCommand::NavigateBack }
            else if action == sel!(shellNavigateForward:) { WindowCommand::NavigateForward }
            else if action == sel!(shellOpenInDefaultBrowser:) { WindowCommand::OpenInDefaultBrowser }
            else if action == sel!(shellToggleSidebar:) { WindowCommand::ToggleSidebar }
            else if action == sel!(shellShowOptions:) { WindowCommand::ShowOptions }
            else if action == sel!(shellToggleOptionShowLogs:) { WindowCommand::ToggleOptionShowLogs }
            else if action == sel!(shellToggleOptionFragmentBorders:) { WindowCommand::ToggleOptionFragmentBorders }
            else if action == sel!(shellToggleOptionParallelDisplayListBuidling:) { WindowCommand::ToggleOptionParallelDisplayListBuidling }
            else if action == sel!(shellToggleOptionShowParallelLayout:) { WindowCommand::ToggleOptionShowParallelLayout }
            else if action == sel!(shellToggleOptionConvertMouseToTouch:) { WindowCommand::ToggleOptionConvertMouseToTouch }
            else if action == sel!(shellToggleOptionTileBorders:) { WindowCommand::ToggleOptionTileBorders }
            else if action == sel!(shellToggleOptionWRProfiler:) { WindowCommand::ToggleOptionWRProfiler }
            else if action == sel!(shellToggleOptionWRTextureCacheDebug:) { WindowCommand::ToggleOptionWRTextureCacheDebug }
            else if action == sel!(shellToggleOptionWRRenderTargetDebug:) { WindowCommand::ToggleOptionWRTargetDebug }
            else {
                panic!("Unexpected action to record: {:?}", action)
            };
            utils::get_event_queue(this).push(WindowEvent::DoCommand(cmd));
        }

        extern fn validate_ui(this: &Object, _sel: Sel, item: id) -> BOOL {
            unsafe {
                let action: id = msg_send![item, action];
                msg_send![this, validateAction:action]
            }
        }

        extern fn validate_action(_this: &Object, _sel: Sel, action: Sel) -> BOOL {
            let idx = get_state().windows[0].current_browser_index.unwrap();
            let ref state = get_state().windows[0].browsers[idx];
            let enabled = if action == sel!(shellStop:) {
                state.is_loading
            } else if action == sel!(shellReload:) {
                !state.is_loading
            } else if action == sel!(shellOpenLocation:) {
                true
            } else if action == sel!(shellNewTab:) {
                true
            } else if action == sel!(shellCloseTab:) {
                true
            } else if action == sel!(shellNextTab:) {
                true
            } else if action == sel!(shellPrevTab:) {
                true
            } else if action == sel!(shellZoomIn:) {
                true
            } else if action == sel!(shellZoomOut:) {
                true
            } else if action == sel!(shellZoomToActualSize:) {
                state.zoom != 1.0
            } else if action == sel!(shellNavigateBack:) {
                state.can_go_back
            } else if action == sel!(shellNavigateForward:) {
                state.can_go_forward
            } else if action == sel!(shellOpenInDefaultBrowser:) {
                match state.url {
                    Some(ref url) if url != "about:blank" => true,
                    _ => false
                }
            } else if action == sel!(shellToggleSidebar:) {
                true
            } else if action == sel!(shellShowOptions:) {
                true
            } else if action == sel!(shellSubmitUserInput:) {
                true
            } else {
                panic!("Unexpected action to validate: {:?}", action);
            };
            if enabled {YES} else {NO}
        }

        extern fn get_state_for_action(_this: &Object, _sel: Sel, action: Sel) -> NSInteger {
            let debug_options = &get_state().windows[0].debug_options;
            let on = if action == sel!(shellToggleOptionDarkTheme:) {
                get_state().dark_theme
            } else if action == sel!(shellToggleOptionShowLogs:) {
                get_state().windows[0].logs_visible
            } else if action == sel!(shellToggleOptionFragmentBorders:) {
                debug_options.show_fragment_borders
            } else if action == sel!(shellToggleOptionParallelDisplayListBuidling:) {
                debug_options.parallel_display_list_building
            } else if action == sel!(shellToggleOptionShowParallelLayout:) {
                debug_options.show_parallel_layout
            } else if action == sel!(shellToggleOptionConvertMouseToTouch:) {
                debug_options.convert_mouse_to_touch
            } else if action == sel!(shellToggleOptionTileBorders:) {
                debug_options.show_tiles_borders
            } else if action == sel!(shellToggleOptionWRProfiler:) {
                debug_options.wr_profiler
            } else if action == sel!(shellToggleOptionWRTextureCacheDebug:) {
                debug_options.wr_texture_cache_debug
            } else if action == sel!(shellToggleOptionWRRenderTargetDebug:) {
                debug_options.wr_render_target_debug
            } else {
                panic!("Unexpected action for getStateForAction: {:?}", action);
            };
            if on {1} else {0}
        }

        extern fn submit_user_input(this: &Object, _sel: Sel, item: id) {
            let text = unsafe {
                let text: id = msg_send![item, stringValue];
                let text: *const libc::c_char = msg_send![text, UTF8String];
                CStr::from_ptr(text).to_string_lossy().into_owned()
            };
            let cmd = WindowCommand::Load(text);
            utils::get_event_queue(this).push(WindowEvent::DoCommand(cmd));
        }

        extern fn tabview_selected(this: &Object, _sel: Sel, tabview: id, item: id) {
            unsafe {
                let rendering: bool = *this.get_ivar("rendering");
                if !rendering {
                    let idx: NSInteger = msg_send![tabview, indexOfTabViewItem:item];
                    let cmd = WindowCommand::SelectTab(idx as usize);
                    utils::get_event_queue(this).push(WindowEvent::DoCommand(cmd));
                }
            };
        }

        unsafe {
            // We don't need to record the windowDidResize notification as the view does record the
            // viewDidEndLiveResize notification.
            // class.add_method(sel!(windowDidResize:), record_notification as extern fn(&Object, Sel, id));

            class.add_method(sel!(windowDidEnterFullScreen:), record_notification as extern fn(&Object, Sel, id));
            class.add_method(sel!(windowDidExitFullScreen:), record_notification as extern fn(&Object, Sel, id));
            class.add_method(sel!(windowWillClose:), record_notification as extern fn(&Object, Sel, id));

            class.add_method(sel!(shellStop:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellReload:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellOpenLocation:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellNewTab:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellCloseTab:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellNextTab:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellPrevTab:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellZoomIn:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellZoomOut:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellZoomToActualSize:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellNavigateBack:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellNavigateForward:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellOpenInDefaultBrowser:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellToggleSidebar:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellShowOptions:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellToggleOptionShowLogs:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellToggleOptionFragmentBorders:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellToggleOptionParallelDisplayListBuidling:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellToggleOptionShowParallelLayout:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellToggleOptionConvertMouseToTouch:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellToggleOptionWRProfiler:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellToggleOptionWRTextureCacheDebug:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellToggleOptionWRRenderTargetDebug:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellToggleOptionTileBorders:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellZoom:), record_command as extern fn(&Object, Sel, id));
            class.add_method(sel!(shellNavigate:), record_command as extern fn(&Object, Sel, id));

            class.add_method(sel!(validateUserInterfaceItem:), validate_ui as extern fn(&Object, Sel, id) -> BOOL);
            class.add_method(sel!(validateAction:), validate_action as extern fn(&Object, Sel, Sel) -> BOOL);
            class.add_method(sel!(getStateForAction:), get_state_for_action as extern fn(&Object, Sel, Sel) -> NSInteger);

            class.add_method(sel!(shellSubmitUserInput:), submit_user_input as extern fn(&Object, Sel, id));

            class.add_method(sel!(tabView:didSelectTabViewItem:), tabview_selected as extern fn(&Object, Sel, id, id));
        }

        class.register();
    }
}


pub struct Window {
    nswindow: id,
    nspopover: id,
}

impl Window {
    pub fn new(nswindow: id, nspopover: id) -> Window {

        unsafe {
            // FIXME: release and set delegate to nil
            let delegate: id = msg_send![class("NSShellWindowDelegate"), alloc];
            let event_queue_ptr: *mut c_void = *(&*nswindow).get_ivar("event_queue");
            (*delegate).set_ivar("event_queue", event_queue_ptr);
            (*delegate).set_ivar("rendering", true);

            msg_send![nswindow, setDelegate:delegate];

            msg_send![nspopover, setBehavior:1]; // NSPopoverBehaviorTransient

            nswindow.setTitleVisibility_(NSWindowTitleVisibility::NSWindowTitleHidden);
            nswindow.setAcceptsMouseMovedEvents_(YES);

            let toolbar: id = msg_send![nswindow, toolbar];
            msg_send![toolbar, setShowsBaselineSeparator:NO];

            let tabbar = utils::get_view_by_id(nswindow, "tabbar").unwrap();
            msg_send![tabbar, setStyleNamed:NSString::alloc(nil).init_str("Yosemite")];
            msg_send![tabbar, setCanCloseOnlyTab:YES];
            msg_send![tabbar, setDisableTabClose:NO];
            msg_send![tabbar, setAllowsBackgroundTabClosing:YES];
            msg_send![tabbar, setHideForSingleTab:YES];
            msg_send![tabbar, setShowAddTabButton:YES];
            msg_send![tabbar, setUseOverflowMenu:YES];
            msg_send![tabbar, setSizeButtonsToFit:NO];
            msg_send![tabbar, setButtonMinWidth:100];
            msg_send![tabbar, setButtonOptimumWidth:200];
            msg_send![tabbar, setButtonMaxWidth:300];
            msg_send![tabbar, setAutomaticallyAnimates:YES];

            msg_send![tabbar, setDelegate:delegate];

            // Necessary to prevent the log view to wrap text
            let textview = utils::get_view_by_id(nswindow, "shellViewLogsTextView").unwrap();
            let text_container: id = msg_send![textview, textContainer];
            msg_send![text_container, setWidthTracksTextView:NO];
            msg_send![text_container, setContainerSize:NSSize::new(f64::MAX, f64::MAX)];
            (*delegate).set_ivar("rendering", false);
        }

        Window {
            nswindow: nswindow,
            nspopover: nspopover,
        }
    }

    pub fn state_changed(&self) {

        // FIXME: long function is long

        self.update_theme();

        let delegate = unsafe {
            let delegate: id = msg_send![self.nswindow, delegate];
            (*delegate).set_ivar("rendering", true);
            delegate
        };

        // First, update the avaibility of the buttons
        unsafe {
            let toolbar: id = msg_send![self.nswindow, toolbar];
            let items: id = msg_send![toolbar, items];
            let count: NSInteger = msg_send![items, count];
            for i in 0..count {
                let item: id = msg_send![items, objectAtIndex:i];
                let view: id = msg_send![item, view];
                if view == nil {
                    continue;
                }
                let action: Sel = msg_send![item, action];
                let identifier: id = msg_send![view, identifier];
                if NSString::isEqualToString(identifier, "shellToolbarViewUrlbar") {
                    let stopped: BOOL = msg_send![delegate, validateAction:sel!(shellStop:)];
                    let indicator = utils::get_view_by_id(view, "shellToolbarViewUrlbarThrobber").unwrap();
                    if stopped == YES {
                        msg_send![indicator, startAnimation:nil];
                    } else {
                        msg_send![indicator, stopAnimation:nil];
                    }
                } else if action == sel!(shellNavigate:) {
                    let enabled0: BOOL = msg_send![delegate, validateAction:sel!(shellNavigateBack:)];
                    let enabled1: BOOL = msg_send![delegate, validateAction:sel!(shellNavigateForward:)];
                    msg_send![view, setEnabled:enabled0 forSegment:0];
                    msg_send![view, setEnabled:enabled1 forSegment:1];
                } else if action == sel!(shellZoom:) {
                    let enabled0: BOOL = msg_send![delegate, validateAction:sel!(shellZoomOut:)];
                    let enabled1: BOOL = msg_send![delegate, validateAction:sel!(shellZoomToActualSize:)];
                    let enabled2: BOOL = msg_send![delegate, validateAction:sel!(shellZoomIn:)];
                    msg_send![view, setEnabled:enabled0 forSegment:0];
                    msg_send![view, setEnabled:enabled1 forSegment:1];
                    msg_send![view, setEnabled:enabled2 forSegment:2];
                } else if action == sel!(shellReloadStop:) {
                    let can_reload: BOOL = msg_send![delegate, validateAction:sel!(shellReload:)];
                    let subviews: id = msg_send![view, subviews];
                    let button_reload: id = msg_send![subviews, objectAtIndex:0];
                    let button_stop: id = msg_send![subviews, objectAtIndex:1];
                    if can_reload == YES {
                        msg_send![button_reload, setEnabled:YES];
                        msg_send![button_reload, setHidden:NO];
                        msg_send![button_stop, setEnabled:NO];
                        msg_send![button_stop, setHidden:YES];
                    } else {
                        msg_send![button_reload, setEnabled:NO];
                        msg_send![button_reload, setHidden:YES];
                        msg_send![button_stop, setEnabled:YES];
                        msg_send![button_stop, setHidden:NO];
                    }
                } else {
                    let enabled: BOOL = msg_send![delegate, validateAction:action];
                    msg_send![view, setEnabled:enabled];
                }
            }
        }

        // Then, update the state of the popover
        unsafe {
            let controller: id = msg_send![self.nspopover, contentViewController];
            let topview: id = msg_send![controller, view];
            let subviews: id = msg_send![topview, subviews];
            let stack: id = msg_send![subviews, objectAtIndex:0];
            let views: id = msg_send![stack, subviews];
            let count: NSInteger = msg_send![views, count];
            for i in 0..count {
                let view: id = msg_send![views, objectAtIndex:i];
                // FIXME
                if utils::id_is_instance_of(view, "NSButton") {
                    let action: Sel = msg_send![view, action];
                    let state: NSInteger = msg_send![delegate, getStateForAction:action];
                    msg_send![view, setState:state];
                }
            }
        }

        // Show logs if necessary
        let logs = utils::get_view_by_id(self.nswindow, "shellViewLogs").unwrap();
        let visible = get_state().windows[0].logs_visible;
        let hidden = if visible {NO} else {YES};
        unsafe {msg_send![logs, setHidden:hidden]};

        // Update urlbar
        let item = self.get_toolbar_item("urlbar").unwrap();
        let idx = get_state().windows[0].current_browser_index.unwrap();
        // FIXME: alloc everytime is bad. Use diff
        unsafe {
            let view = msg_send![item, view];
            let field = utils::get_view_by_id(view, "shellToolbarViewUrlbarTextfield").unwrap();
            match get_state().windows[0].browsers[idx].url {
                Some(ref url) if url != "about:blank" => msg_send![field, setStringValue:NSString::alloc(nil).init_str(url)],
                _ => msg_send![field, setStringValue:NSString::alloc(nil).init_str("")],
            };

            if get_state().windows[0].urlbar_focused {
                msg_send![field, becomeFirstResponder];
                // FIXME: Don't do that!!! We should get a message from the widget
                get_mut_state().windows[0].urlbar_focused = false;
            }
        }

        // FIXME, very ugly
        // Update tabbar
        // Sorry for the basic stupid diff
        let tabview = utils::get_view_by_id(self.nswindow, "tabview").unwrap();

        let visual_count: usize = unsafe { msg_send![tabview, numberOfTabViewItems] };

        unsafe {
            for i in (0..visual_count).rev() {
                let item: id = msg_send![tabview, tabViewItemAtIndex:i];
                let view_id: id = msg_send![item, identifier];
                if !get_state().windows[0].browsers.iter().any(|b| {
                    // FIXME: store String
                    NSString::isEqualToString(view_id, &format!("{}", b.id))
                }) {
                    msg_send![tabview, removeTabViewItem:item];
                }
            }
        }

        let visual_count: usize = unsafe { msg_send![tabview, numberOfTabViewItems] };

        let state_count = get_state().windows[0].browsers.len();

        if state_count == visual_count + 1 {
            // Need to add a tab
            // Always assume extra tab has been added at the end
            let id = get_state().windows[0].browsers[state_count - 1].id;
            unsafe {
                let item: id = msg_send![class("NSTabViewItem"), alloc];
                let identifier = NSString::alloc(nil).init_str(format!("{}", id).as_str());
                let item: id = msg_send![item, initWithIdentifier:identifier];
                msg_send![tabview, addTabViewItem:item];
            }

        } else if state_count != visual_count {
            // That should never happen
            println!("Inconsistent tabs");
        }

        unsafe {
            let idx = get_state().windows[0].current_browser_index.unwrap();
            msg_send![tabview, selectTabViewItemAtIndex:idx];
        }

        unsafe {
            for i in 0..state_count {
                // FIXME: alloc…
                let item: id = msg_send![tabview, tabViewItemAtIndex:i];
                let nsstring = match get_state().windows[0].browsers[i].title {
                    Some(ref title) => NSString::alloc(nil).init_str(title),
                    None => NSString::alloc(nil).init_str("No Title"),
                };
                msg_send![item, setLabel:nsstring];
            }
        }


        // FIXME: This is too basic. If we want animations and proper sidebar support,
        // we need to have access to "animator()" which, afaiu, comes only
        // from a NSSplitViewController. We want to be able to use this:
        // https://developer.apple.com/reference/appkit/nssplitviewcontroller/1388905-togglesidebar
        let sidebar = utils::get_view_by_id(self.nswindow, "shellViewSidebar").unwrap();
        unsafe {
            let hidden = if get_state().windows[0].sidebar_is_open {NO} else {YES};
            msg_send![sidebar, setHidden:hidden];
        }

        // FIXME: diff
        let textfield = utils::get_view_by_id(self.nswindow, "shellStatusLabel").unwrap();
        match get_state().windows[0].status {
            Some(ref status) => {
                unsafe {
                    msg_send![textfield, setHidden:NO];
                    let string = NSString::alloc(nil).init_str(status);
                    NSTextField::setStringValue_(textfield, string);
                }
            }
            None => {
                unsafe{msg_send![textfield, setHidden:YES]};
            }
        }

        unsafe {
            if get_state().windows[0].options_open {
                let item = self.get_toolbar_item("options").unwrap();
                let button: id = msg_send![item, view];
                let bounds = NSView::bounds(button);
                msg_send![self.nspopover, showRelativeToRect:bounds ofView:button preferredEdge:3];
                // FIXME: Don't do that!!! We should get a message from the widget
                get_mut_state().windows[0].options_open = false;
            }
        }

        unsafe {
            (*delegate).set_ivar("rendering", false);
        }

    }

    pub fn get_init_state() -> WindowState {
        WindowState {
            current_browser_index: None,
            browsers: Vec::new(),
            sidebar_is_open: false,
            logs_visible: false,
            status: None,
            urlbar_focused: false,
            options_open: false,
            debug_options: DebugOptions {
                show_fragment_borders: false,
                parallel_display_list_building: false,
                show_parallel_layout: false,
                convert_mouse_to_touch: false,
                show_tiles_borders: false,
                wr_profiler: false,
                wr_texture_cache_debug: false,
                wr_render_target_debug: false,
            },
        }
    }

    pub fn create_view(&self) -> Result<View, &'static str> {
        // FIXME: We should dynamically create a NSServoView,
        // and adds the constraints, instead on relying on IB's instance.
        utils::get_view_by_id(self.nswindow, "shellViewServo")
            .map(|nsview| View::new(nsview))
            .ok_or("Can't find NSServoView")
    }

    pub fn append_logs(&self, logs: &Vec<ShellLog>) {
        unsafe {
            let textview = utils::get_view_by_id(self.nswindow, "shellViewLogsTextView").unwrap();
            let textstorage: id = msg_send![textview, textStorage];
            // FIXME: figure out how to add colors
            for l in logs {
                let mutable_string: id = msg_send![textstorage, mutableString];
                let message = format!("\n{} - {}: {}", l.level, l.target, l.message);
                let message = NSString::alloc(nil).init_str(message.as_str());
                msg_send![mutable_string, appendString:message];
            }
        }
    }

    pub fn get_events(&self) -> Vec<WindowEvent> {
        let nsobject = unsafe { &*self.nswindow};
        utils::get_event_queue(nsobject).drain(..).collect()
    }

    fn get_toolbar_item(&self, identifier: &str) -> Option<id> {
        unsafe {
            let toolbar: id = msg_send![self.nswindow, toolbar];
            let items: id = msg_send![toolbar, items];
            let count: NSInteger = msg_send![items, count];
            for i in 0..count {
                let item: id = msg_send![items, objectAtIndex:i];
                let item_identifier: id = msg_send![item, itemIdentifier];
                if NSString::isEqualToString(item_identifier, identifier) {
                    return Some(item);
                }
            }
            None
        }
    }

    pub fn set_title(&self, title: &str) {
        // FIXME: never called
        unsafe {
            let title = NSString::alloc(nil).init_str(title);
            msg_send![self.nswindow, setTitle:title]
        }
    }

    pub fn create_event_loop_waker(&self) -> Box<EventLoopWaker> {
        let window_number: NSInteger = unsafe {
            msg_send![self.nswindow, windowNumber]
        };
        box MacOSEventLoopWaker {
            window_number: window_number,
        }
    }

    fn update_theme(&self) {
        let was_dark = {
            // FIXME. Use state diff
            unsafe {
                let appearance: id = msg_send![self.nswindow, appearance];
                let name: id = msg_send![appearance, name];
                name == NSAppearanceNameVibrantDark
            }
        };
        let dark = get_state().dark_theme;

        if (dark && was_dark) || (!dark && !was_dark) {
            return
        }

        let (appearance, bordered, segment_style) = unsafe { if dark {
            // 3 -> roundRect
            (NSAppearanceNameVibrantDark, NO, 3)
        } else {
            // 0 -> automatic
            (NSAppearanceNameVibrantLight, YES, 0)
        }};

        let item = self.get_toolbar_item("options").unwrap();
        let topview = unsafe {
            let view: id = msg_send![item, view];
            let view: id = msg_send![view, superview];
            msg_send![view, superview]
        };
        utils::get_view(topview, &|view| {
            if utils::id_is_instance_of(view, "NSButton") {
                unsafe {msg_send![view, setBordered:bordered]};
            }
            if utils::id_is_instance_of(view, "NSSegmentedControl") {
                unsafe {msg_send![view, setSegmentStyle:segment_style]};
            }
            if utils::id_is_instance_of(view, "NSTextField") {
                unsafe {
                    let layer: id = msg_send![view, layer];
                    msg_send![layer, setCornerRadius:3.0];
                    let alpha = if dark {0.1} else {0.0};
                    let color: id = msg_send![Class::get("NSColor").unwrap(), colorWithRed:1.0 green:1.0 blue:1.0 alpha:alpha];
                    let color: id = msg_send![color, CGColor];
                    msg_send![layer, setBackgroundColor:color];
                }
            }
            false
        });
        unsafe {
            let appearance: id = msg_send![class("NSAppearance"), appearanceNamed:appearance];
            msg_send![self.nswindow, setAppearance:appearance];
        }
    }
}

pub struct MacOSEventLoopWaker {
    window_number: NSInteger,
}

impl EventLoopWaker for MacOSEventLoopWaker {
    fn wake(&self) {
        unsafe {
            let pool = NSAutoreleasePool::new(nil);
            let event: id = msg_send![class("NSEvent"),
                    otherEventWithType:NSApplicationDefined
                    location:NSPoint::new(0.0, 0.0)
                    modifierFlags:NSEventModifierFlags::empty()
                    timestamp:0.0
                    windowNumber:self.window_number
                    context:nil
                    subtype:NSEventSubtype::NSApplicationActivatedEventType
                    data1:0
                    data2:0];
            msg_send![event, retain];
            msg_send![NSApp(), postEvent:event atStart:NO];
            NSAutoreleasePool::drain(pool);
        }
    }
    fn clone(&self) -> Box<EventLoopWaker + Send> {
        box MacOSEventLoopWaker {
            window_number: self.window_number,
        }
    }
}
