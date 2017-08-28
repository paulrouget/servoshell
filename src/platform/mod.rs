/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub use self::platform::*;

#[cfg(feature = "with-cocoa")]
#[path="cocoa/mod.rs"]
mod platform;

#[cfg(feature = "with-glutin")]
#[path="glutin/mod.rs"]
mod platform;
