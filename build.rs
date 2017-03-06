extern crate fs_extra;

use fs_extra::dir::copy;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;


// #[cfg(target_os = "macos")]

fn main() {
    let target_dir = env::var("CARGO_TARGET_DIR").unwrap();
    let app_dir = Path::new(&target_dir).join("Servo Shell");
    let bin_dir = app_dir.join("Contents").join("MacOS");
    let res_dir = app_dir.join("Resources");

    fs::create_dir_all(&bin_dir).unwrap();
    fs::create_dir_all(&res_dir).unwrap();

    let options = dir::CopyOptions::new();
    fs_extra::dir::copy_items(Vec::new(vec!["resources"]), &res_dir, &options)?;

    let res_dir_str = res_dir.to_str().unwrap();
    ibtool("macos/xib/App.xib", res_dir_str);
    ibtool("macos/xib/Window.xib", res_dir_str);

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
