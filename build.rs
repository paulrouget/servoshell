/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    build_mmtabbarview();
    build_nibs();
}

fn build_mmtabbarview() {
    Command::new("xcodebuild")
        .args(&["-project", "./macos/MMTabBarView/MMTabBarView/MMTabBarView.xcodeproj"])
        .args(&["-configuration", "Release"])
        .args(&["SYMROOT=../../../target/MMTabBarView/"])
        .status()
        .expect("xcodebuild failed");
    println!("cargo:rustc-link-search=framework=target/MMTabBarView/Release/");
}

fn build_nibs() {
    fn ibtool(src: &str, out_dir: &Path) {
        let out = out_dir.to_str().unwrap();
        let filename = Path::new(src).file_name().unwrap();
        let out_file = filename.to_str().unwrap().replace("xib", "nib");
        Command::new("ibtool")
            .arg(src)
            .arg("--compile")
            .arg(&format!("{}/{}", out, out_file))
            .status()
            .ok()
            .expect("ibtool failed");
    }
    let nibs_dir = Path::new("nibs");
    fs::create_dir_all(&nibs_dir).unwrap();
    ibtool("macos/xib/App.xib", nibs_dir);
    ibtool("macos/xib/Window.xib", nibs_dir);
}
