/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use glutin;
use servo::ServoCursor;
use traits::view::Key;

pub fn glutin_key_to_script_key(key: glutin::VirtualKeyCode) -> Result<Key, ()> {
    match key {
        glutin::VirtualKeyCode::A => Ok(Key::A),
        glutin::VirtualKeyCode::B => Ok(Key::B),
        glutin::VirtualKeyCode::C => Ok(Key::C),
        glutin::VirtualKeyCode::D => Ok(Key::D),
        glutin::VirtualKeyCode::E => Ok(Key::E),
        glutin::VirtualKeyCode::F => Ok(Key::F),
        glutin::VirtualKeyCode::G => Ok(Key::G),
        glutin::VirtualKeyCode::H => Ok(Key::H),
        glutin::VirtualKeyCode::I => Ok(Key::I),
        glutin::VirtualKeyCode::J => Ok(Key::J),
        glutin::VirtualKeyCode::K => Ok(Key::K),
        glutin::VirtualKeyCode::L => Ok(Key::L),
        glutin::VirtualKeyCode::M => Ok(Key::M),
        glutin::VirtualKeyCode::N => Ok(Key::N),
        glutin::VirtualKeyCode::O => Ok(Key::O),
        glutin::VirtualKeyCode::P => Ok(Key::P),
        glutin::VirtualKeyCode::Q => Ok(Key::Q),
        glutin::VirtualKeyCode::R => Ok(Key::R),
        glutin::VirtualKeyCode::S => Ok(Key::S),
        glutin::VirtualKeyCode::T => Ok(Key::T),
        glutin::VirtualKeyCode::U => Ok(Key::U),
        glutin::VirtualKeyCode::V => Ok(Key::V),
        glutin::VirtualKeyCode::W => Ok(Key::W),
        glutin::VirtualKeyCode::X => Ok(Key::X),
        glutin::VirtualKeyCode::Y => Ok(Key::Y),
        glutin::VirtualKeyCode::Z => Ok(Key::Z),

        glutin::VirtualKeyCode::Numpad0 => Ok(Key::Kp0),
        glutin::VirtualKeyCode::Numpad1 => Ok(Key::Kp1),
        glutin::VirtualKeyCode::Numpad2 => Ok(Key::Kp2),
        glutin::VirtualKeyCode::Numpad3 => Ok(Key::Kp3),
        glutin::VirtualKeyCode::Numpad4 => Ok(Key::Kp4),
        glutin::VirtualKeyCode::Numpad5 => Ok(Key::Kp5),
        glutin::VirtualKeyCode::Numpad6 => Ok(Key::Kp6),
        glutin::VirtualKeyCode::Numpad7 => Ok(Key::Kp7),
        glutin::VirtualKeyCode::Numpad8 => Ok(Key::Kp8),
        glutin::VirtualKeyCode::Numpad9 => Ok(Key::Kp9),

        glutin::VirtualKeyCode::Key0 => Ok(Key::Num0),
        glutin::VirtualKeyCode::Key1 => Ok(Key::Num1),
        glutin::VirtualKeyCode::Key2 => Ok(Key::Num2),
        glutin::VirtualKeyCode::Key3 => Ok(Key::Num3),
        glutin::VirtualKeyCode::Key4 => Ok(Key::Num4),
        glutin::VirtualKeyCode::Key5 => Ok(Key::Num5),
        glutin::VirtualKeyCode::Key6 => Ok(Key::Num6),
        glutin::VirtualKeyCode::Key7 => Ok(Key::Num7),
        glutin::VirtualKeyCode::Key8 => Ok(Key::Num8),
        glutin::VirtualKeyCode::Key9 => Ok(Key::Num9),

        glutin::VirtualKeyCode::Return => Ok(Key::Enter),
        glutin::VirtualKeyCode::Space => Ok(Key::Space),
        glutin::VirtualKeyCode::Escape => Ok(Key::Escape),
        glutin::VirtualKeyCode::Equals => Ok(Key::Equal),
        glutin::VirtualKeyCode::Minus => Ok(Key::Minus),
        glutin::VirtualKeyCode::Back => Ok(Key::Backspace),
        glutin::VirtualKeyCode::PageDown => Ok(Key::PageDown),
        glutin::VirtualKeyCode::PageUp => Ok(Key::PageUp),

        glutin::VirtualKeyCode::Insert => Ok(Key::Insert),
        glutin::VirtualKeyCode::Home => Ok(Key::Home),
        glutin::VirtualKeyCode::Delete => Ok(Key::Delete),
        glutin::VirtualKeyCode::End => Ok(Key::End),

        glutin::VirtualKeyCode::Left => Ok(Key::Left),
        glutin::VirtualKeyCode::Up => Ok(Key::Up),
        glutin::VirtualKeyCode::Right => Ok(Key::Right),
        glutin::VirtualKeyCode::Down => Ok(Key::Down),

        glutin::VirtualKeyCode::LShift => Ok(Key::LeftShift),
        glutin::VirtualKeyCode::LControl => Ok(Key::LeftControl),
        glutin::VirtualKeyCode::LAlt => Ok(Key::LeftAlt),
        glutin::VirtualKeyCode::LWin => Ok(Key::LeftSuper),
        glutin::VirtualKeyCode::RShift => Ok(Key::RightShift),
        glutin::VirtualKeyCode::RControl => Ok(Key::RightControl),
        glutin::VirtualKeyCode::RAlt => Ok(Key::RightAlt),
        glutin::VirtualKeyCode::RWin => Ok(Key::RightSuper),

        glutin::VirtualKeyCode::Apostrophe => Ok(Key::Apostrophe),
        glutin::VirtualKeyCode::Backslash => Ok(Key::Backslash),
        glutin::VirtualKeyCode::Comma => Ok(Key::Comma),
        glutin::VirtualKeyCode::Grave => Ok(Key::GraveAccent),
        glutin::VirtualKeyCode::LBracket => Ok(Key::LeftBracket),
        glutin::VirtualKeyCode::Period => Ok(Key::Period),
        glutin::VirtualKeyCode::RBracket => Ok(Key::RightBracket),
        glutin::VirtualKeyCode::Semicolon => Ok(Key::Semicolon),
        glutin::VirtualKeyCode::Slash => Ok(Key::Slash),
        glutin::VirtualKeyCode::Tab => Ok(Key::Tab),
        glutin::VirtualKeyCode::Subtract => Ok(Key::Minus),

        glutin::VirtualKeyCode::F1 => Ok(Key::F1),
        glutin::VirtualKeyCode::F2 => Ok(Key::F2),
        glutin::VirtualKeyCode::F3 => Ok(Key::F3),
        glutin::VirtualKeyCode::F4 => Ok(Key::F4),
        glutin::VirtualKeyCode::F5 => Ok(Key::F5),
        glutin::VirtualKeyCode::F6 => Ok(Key::F6),
        glutin::VirtualKeyCode::F7 => Ok(Key::F7),
        glutin::VirtualKeyCode::F8 => Ok(Key::F8),
        glutin::VirtualKeyCode::F9 => Ok(Key::F9),
        glutin::VirtualKeyCode::F10 => Ok(Key::F10),
        glutin::VirtualKeyCode::F11 => Ok(Key::F11),
        glutin::VirtualKeyCode::F12 => Ok(Key::F12),

        glutin::VirtualKeyCode::NavigateBackward => Ok(Key::NavigateBackward),
        glutin::VirtualKeyCode::NavigateForward => Ok(Key::NavigateForward),
        _ => Err(()),
    }
}

pub fn is_printable(key_code: glutin::VirtualKeyCode) -> bool {
    use glutin::VirtualKeyCode::*;
    match key_code {
        Escape | F1 | F2 | F3 | F4 | F5 | F6 | F7 | F8 | F9 | F10 | F11 | F12 | F13 | F14 |
        F15 | Snapshot | Scroll | Pause | Insert | Home | Delete | End | PageDown | PageUp |
        Left | Up | Right | Down | Back | LAlt | LControl | LMenu | LShift | LWin | Mail |
        MediaSelect | MediaStop | Mute | MyComputer | NavigateForward | NavigateBackward |
        NextTrack | NoConvert | PlayPause | Power | PrevTrack | RAlt | RControl | RMenu |
        RShift | RWin | Sleep | Stop | VolumeDown | VolumeUp | Wake | WebBack | WebFavorites |
        WebForward | WebHome | WebRefresh | WebSearch | WebStop => false,
        _ => true,
    }
}

pub fn servo_cursor_to_glutin_cursor(servo_cursor: ServoCursor) -> glutin::MouseCursor {
    match servo_cursor {
        ServoCursor::None => glutin::MouseCursor::NoneCursor,
        ServoCursor::Default => glutin::MouseCursor::Default,
        ServoCursor::Pointer => glutin::MouseCursor::Hand,
        ServoCursor::ContextMenu => glutin::MouseCursor::ContextMenu,
        ServoCursor::Help => glutin::MouseCursor::Help,
        ServoCursor::Progress => glutin::MouseCursor::Progress,
        ServoCursor::Wait => glutin::MouseCursor::Wait,
        ServoCursor::Cell => glutin::MouseCursor::Cell,
        ServoCursor::Crosshair => glutin::MouseCursor::Crosshair,
        ServoCursor::Text => glutin::MouseCursor::Text,
        ServoCursor::VerticalText => glutin::MouseCursor::VerticalText,
        ServoCursor::Alias => glutin::MouseCursor::Alias,
        ServoCursor::Copy => glutin::MouseCursor::Copy,
        ServoCursor::Move => glutin::MouseCursor::Move,
        ServoCursor::NoDrop => glutin::MouseCursor::NoDrop,
        ServoCursor::NotAllowed => glutin::MouseCursor::NotAllowed,
        ServoCursor::Grab => glutin::MouseCursor::Grab,
        ServoCursor::Grabbing => glutin::MouseCursor::Grabbing,
        ServoCursor::EResize => glutin::MouseCursor::EResize,
        ServoCursor::NResize => glutin::MouseCursor::NResize,
        ServoCursor::NeResize => glutin::MouseCursor::NeResize,
        ServoCursor::NwResize => glutin::MouseCursor::NwResize,
        ServoCursor::SResize => glutin::MouseCursor::SResize,
        ServoCursor::SeResize => glutin::MouseCursor::SeResize,
        ServoCursor::SwResize => glutin::MouseCursor::SwResize,
        ServoCursor::WResize => glutin::MouseCursor::WResize,
        ServoCursor::EwResize => glutin::MouseCursor::EwResize,
        ServoCursor::NsResize => glutin::MouseCursor::NsResize,
        ServoCursor::NeswResize => glutin::MouseCursor::NeswResize,
        ServoCursor::NwseResize => glutin::MouseCursor::NwseResize,
        ServoCursor::ColResize => glutin::MouseCursor::ColResize,
        ServoCursor::RowResize => glutin::MouseCursor::RowResize,
        ServoCursor::AllScroll => glutin::MouseCursor::AllScroll,
        ServoCursor::ZoomIn => glutin::MouseCursor::ZoomIn,
        ServoCursor::ZoomOut => glutin::MouseCursor::ZoomOut,
    }
}

// Some shortcuts use Cmd on Mac and Control on other systems.
pub fn cmd_or_ctrl(m: glutin::ModifiersState) -> bool {
    if cfg!(target_os = "macos") {
        m.logo
    } else {
        m.ctrl
    }
}


pub fn char_to_script_key(c: char) -> Option<Key> {
    match c {
        ' ' => Some(Key::Space),
        '"' => Some(Key::Apostrophe),
        '\'' => Some(Key::Apostrophe),
        '<' => Some(Key::Comma),
        ',' => Some(Key::Comma),
        '_' => Some(Key::Minus),
        '-' => Some(Key::Minus),
        '>' => Some(Key::Period),
        '.' => Some(Key::Period),
        '?' => Some(Key::Slash),
        '/' => Some(Key::Slash),
        '~' => Some(Key::GraveAccent),
        '`' => Some(Key::GraveAccent),
        ')' => Some(Key::Num0),
        '0' => Some(Key::Num0),
        '!' => Some(Key::Num1),
        '1' => Some(Key::Num1),
        '@' => Some(Key::Num2),
        '2' => Some(Key::Num2),
        '#' => Some(Key::Num3),
        '3' => Some(Key::Num3),
        '$' => Some(Key::Num4),
        '4' => Some(Key::Num4),
        '%' => Some(Key::Num5),
        '5' => Some(Key::Num5),
        '^' => Some(Key::Num6),
        '6' => Some(Key::Num6),
        '&' => Some(Key::Num7),
        '7' => Some(Key::Num7),
        '*' => Some(Key::Num8),
        '8' => Some(Key::Num8),
        '(' => Some(Key::Num9),
        '9' => Some(Key::Num9),
        ':' => Some(Key::Semicolon),
        ';' => Some(Key::Semicolon),
        '+' => Some(Key::Equal),
        '=' => Some(Key::Equal),
        'A' => Some(Key::A),
        'a' => Some(Key::A),
        'B' => Some(Key::B),
        'b' => Some(Key::B),
        'C' => Some(Key::C),
        'c' => Some(Key::C),
        'D' => Some(Key::D),
        'd' => Some(Key::D),
        'E' => Some(Key::E),
        'e' => Some(Key::E),
        'F' => Some(Key::F),
        'f' => Some(Key::F),
        'G' => Some(Key::G),
        'g' => Some(Key::G),
        'H' => Some(Key::H),
        'h' => Some(Key::H),
        'I' => Some(Key::I),
        'i' => Some(Key::I),
        'J' => Some(Key::J),
        'j' => Some(Key::J),
        'K' => Some(Key::K),
        'k' => Some(Key::K),
        'L' => Some(Key::L),
        'l' => Some(Key::L),
        'M' => Some(Key::M),
        'm' => Some(Key::M),
        'N' => Some(Key::N),
        'n' => Some(Key::N),
        'O' => Some(Key::O),
        'o' => Some(Key::O),
        'P' => Some(Key::P),
        'p' => Some(Key::P),
        'Q' => Some(Key::Q),
        'q' => Some(Key::Q),
        'R' => Some(Key::R),
        'r' => Some(Key::R),
        'S' => Some(Key::S),
        's' => Some(Key::S),
        'T' => Some(Key::T),
        't' => Some(Key::T),
        'U' => Some(Key::U),
        'u' => Some(Key::U),
        'V' => Some(Key::V),
        'v' => Some(Key::V),
        'W' => Some(Key::W),
        'w' => Some(Key::W),
        'X' => Some(Key::X),
        'x' => Some(Key::X),
        'Y' => Some(Key::Y),
        'y' => Some(Key::Y),
        'Z' => Some(Key::Z),
        'z' => Some(Key::Z),
        '{' => Some(Key::LeftBracket),
        '[' => Some(Key::LeftBracket),
        '|' => Some(Key::Backslash),
        '\\' => Some(Key::Backslash),
        '}' => Some(Key::RightBracket),
        ']' => Some(Key::RightBracket),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
pub fn windows_hidpi_factor() -> f32 {
    use user32;
    use winapi;
    use gdi32;

    let hdc = unsafe { user32::GetDC(::std::ptr::null_mut()) };
    let ppi = unsafe { gdi32::GetDeviceCaps(hdc, winapi::wingdi::LOGPIXELSY) };
    ppi as f32 / 96.0
}
