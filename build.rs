extern crate fs_extra;

use fs_extra::{copy_items, dir};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;


// #[cfg(target_os = "macos")]

fn main() {
    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let app_dir = Path::new(&out_dir).join("Servo Shell");
    let bin_dir = app_dir.join("Contents").join("MacOS");
    let res_dir = app_dir.join("Resources");

    fs::create_dir_all(&bin_dir).unwrap();
    fs::create_dir_all(&res_dir).unwrap();

    let org_res_dir = Path::new("resources");
    let dir_content = fs::read_dir(org_res_dir).unwrap().map(|e| {
        e.unwrap().path().to_str().unwrap().to_owned()
    }).collect::<Vec<String>>();
    let options = dir::CopyOptions::new();
    copy_items(&dir_content, &res_dir, &options).unwrap();

    let res_dir_str = res_dir.to_str().unwrap();
    ibtool("macos/xib/App.xib", res_dir_str);
    ibtool("macos/xib/Window.xib", res_dir_str);

    let exe_name = Path::new(&out_dir).join("servoshell"));
}

fn ibtool(src: &str, out_dir: &str) {
    let filename = Path::new(src).file_name().unwrap();
    let out_file = filename.to_str().unwrap().replace("xib", "nib");
    Command::new("ibtool")
        .arg(src)
        .arg("--compile")
        .arg(&format!("{}/{}", out_dir, out_file))
        .status()
        .ok()
        .expect("ibtool failed");
}
