/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub use self::platform::*;

#[cfg(all(not(feature = "force-glutin"), target_os = "macos"))]
#[path="cocoa/mod.rs"]
mod platform;

#[cfg(any(feature = "force-glutin", not(target_os = "macos")))]
#[path="glutin/mod.rs"]
mod platform;
