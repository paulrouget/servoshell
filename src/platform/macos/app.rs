use cocoa::appkit::*;
use cocoa::base::*;
use cocoa::foundation::*;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc_foundation::{INSObject, NSObject};

use std::sync::{Once, ONCE_INIT};
static START: Once = ONCE_INIT;

pub fn setup() {
    unsafe {
        START.call_once(|| {
            build_menu();
            register_app_delegate();
            let delegate: id = msg_send![Class::get("AppDelegate").unwrap(), new];
            msg_send![delegate, retain]; // FIXME: release?
            // FIXME: When is setDelegate:nil called???
            msg_send![NSApp(), setDelegate:delegate];
        });
    }
}

fn build_menu() {
    unsafe {
        // Apparently, these selector are used against the window delegate (which is good),
        let quit_item = {
            let label = NSString::alloc(nil).init_str("Quit");
            let action = selector("terminate:");
            let key = NSString::alloc(nil).init_str("q");
            NSMenuItem::alloc(nil)
                .initWithTitle_action_keyEquivalent_(label, action, key)
                .autorelease()
        };

        let about_item = {
            let label = NSString::alloc(nil).init_str("About");
            let action = selector("orderFrontStandardAboutPanel:");
            let key = NSString::alloc(nil).init_str("a");
            NSMenuItem::alloc(nil)
                .initWithTitle_action_keyEquivalent_(label, action, key)
                .autorelease()
        };

        let reload_item = {
            let label = NSString::alloc(nil).init_str("Reload");
            let action = selector("reload");
            let key = NSString::alloc(nil).init_str("r");
            NSMenuItem::alloc(nil)
                .initWithTitle_action_keyEquivalent_(label, action, key)
                .autorelease()
        };

        let go_back_item = {
            let label = NSString::alloc(nil).init_str("Back");
            let action = selector("go_back");
            let key = NSString::alloc(nil).init_str("[");
            NSMenuItem::alloc(nil)
                .initWithTitle_action_keyEquivalent_(label, action, key)
                .autorelease()
        };

        let go_fwd_item = {
            let label = NSString::alloc(nil).init_str("Forward");
            let action = selector("go_forward");
            let key = NSString::alloc(nil).init_str("]");
            NSMenuItem::alloc(nil)
                .initWithTitle_action_keyEquivalent_(label, action, key)
                .autorelease()
        };

        let open_location_item = {
            let label = NSString::alloc(nil).init_str("Open Location");
            let action = selector("open_location");
            let key = NSString::alloc(nil).init_str("l");
            NSMenuItem::alloc(nil)
                .initWithTitle_action_keyEquivalent_(label, action, key)
                .autorelease()
        };


        let app_menu = NSMenu::new(nil).autorelease();
        app_menu.addItem_(quit_item);
        app_menu.addItem_(about_item);
        let app_menu_item = NSMenuItem::new(nil).autorelease();
        app_menu_item.setSubmenu_(app_menu);

        let file_menu =
            NSMenu::new(nil).initWithTitle_(NSString::alloc(nil).init_str("File")).autorelease();
        file_menu.addItem_(open_location_item);
        file_menu.setAutoenablesItems(NO);
        let file_menu_item = NSMenuItem::new(nil).autorelease();
        file_menu_item.setSubmenu_(file_menu);

        let view_menu =
            NSMenu::new(nil).initWithTitle_(NSString::alloc(nil).init_str("View")).autorelease();
        view_menu.addItem_(reload_item);
        view_menu.setAutoenablesItems(NO);
        let view_menu_item = NSMenuItem::new(nil).autorelease();
        view_menu_item.setSubmenu_(view_menu);

        let history_menu =
            NSMenu::new(nil).initWithTitle_(NSString::alloc(nil).init_str("History")).autorelease();
        history_menu.addItem_(go_back_item);
        history_menu.addItem_(go_fwd_item);
        history_menu.setAutoenablesItems(NO);
        let history_menu_item = NSMenuItem::new(nil).autorelease();
        history_menu_item.setSubmenu_(history_menu);

        let menubar = NSMenu::new(nil).autorelease();
        menubar.addItem_(app_menu_item);
        menubar.addItem_(file_menu_item);
        menubar.addItem_(view_menu_item);
        menubar.addItem_(history_menu_item);


        NSApp().setMainMenu_(menubar);

    }
}

fn register_app_delegate() {
    unsafe {
        let superclass = NSObject::class();
        let mut decl = ClassDecl::new("AppDelegate", superclass).unwrap();
        decl.add_method(selector("applicationDidFinishLaunching:"),
                        application_did_finish_launching as extern fn(&Object, Sel, id));
        decl.register();
    }
}

extern fn application_did_finish_launching(_this: &Object, _cmd: Sel, _id: id) {}
