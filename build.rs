/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    if cfg!(all(not(feature = "force-glutin"), target_os = "macos")) {
        build_mmtabbarview();
        build_nibs();
    }
}

fn build_mmtabbarview() {
    if !Path::new("./src/platform/cocoa/MMTabBarView/.git").exists() {
        let _ = Command::new("git")
            .args(&["submodule", "update", "--init"])
            .status();
    }
    let status = Command::new("xcodebuild")
        .args(&["-project",
                "./src/platform/cocoa/MMTabBarView/MMTabBarView/MMTabBarView.xcodeproj"])
        .args(&["-configuration", "Release"])
        .args(&["SYMROOT=../../../../../target/MMTabBarView/"])
        .status()
        .expect("failed to execute xcodebuild");
    assert!(status.success(), "xcodebuild failed");

    println!("cargo:rustc-link-search=framework=target/MMTabBarView/Release/");
}

fn build_nibs() {
    fn ibtool(src: &str, out_dir: &Path) {
        let out = out_dir.to_str().unwrap();
        let filename = Path::new(src).file_name().unwrap();
        let out_file = filename.to_str().unwrap().replace("xib", "nib");
        let status = Command::new("ibtool")
            .arg(src)
            .arg("--compile")
            .arg(&format!("{}/{}", out, out_file))
            .status()
            .expect("failed to execute ibtool");
        assert!(status.success(), "ibtool failed");
    }
    let nibs_dir = Path::new("target/nibs");
    fs::create_dir_all(&nibs_dir).unwrap();
    ibtool("src/platform/cocoa/xib/App.xib", nibs_dir);
    ibtool("src/platform/cocoa/xib/Window.xib", nibs_dir);
}
