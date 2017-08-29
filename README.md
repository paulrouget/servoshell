# servoshell

This is a sandbox project. Prototyping and experimenting with embedding Servo.
The code compiles and run, but it's messy and memory management is non existent.

## Build

You need cargo nightly. Cargo will most likely complain about missing toolchain.
Install it but no needs to make it the default.

You need to pick which version of the UI you want. Either cocoa widgets (only MacOS),
or the fallback Glutin port (Windows, MacOS and Linux).

1. `git clone --recursive` or `git submodule update --init --recursive` if already cloned.
2. `cargo build --release --features=with-cocoa` or `cargo build --release --features=with-glutin`
3. `cargo run --release`

## How to update Servo

1. change `rev` in `Cargo.toml`
2. update `rust-toolchain`
3. copy servo/Cargo.lock to servoshell/Cargo.lock
4. copy servo/resources to servoshell/servo_resources

## Screenshots

![tabs](https://github.com/paulrouget/servoshell/blob/master/screenshots/tabs.png?raw=true "regular")
![regular](https://github.com/paulrouget/servoshell/blob/master/screenshots/regular.png?raw=true "regular")
![dark theme](https://github.com/paulrouget/servoshell/blob/master/screenshots/dark-theme.png?raw=true "dark theme")
![options](https://github.com/paulrouget/servoshell/blob/master/screenshots/options.png?raw=true "options")
![debug](https://github.com/paulrouget/servoshell/blob/master/screenshots/debug.png?raw=true "debug")
