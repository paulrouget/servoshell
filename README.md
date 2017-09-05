# servoshell

This is a sandbox project. Prototyping and experimenting with embedding Servo.
The code compiles and run, but it's messy and memory management is non existent.

## Build

1. ``rustup install `cat rust-toolchain` ``
2. `cargo build --release`
3. `cargo run --release`

There are 2 versions of the UI:
1. Full UI: Tabs + urlbar interface. Cocoa based (only MacOS).
2. Minimal UI: No controls. Driven by keybindings (Windows and Linux).

The minimal UI can be compiled on MacOS with `--features=force-glutin`.

## How to update Servo

1. change `rev` in `Cargo.toml`
2. copy `rust-toolchain` to `servoshell/rust-toolchain`
3. copy `servo/Cargo.lock` to `servoshell/Cargo.lock`
4. copy `servo/resources` to `servoshell/servo_resources`

## Screenshots

![tabs](https://github.com/paulrouget/servoshell/blob/master/screenshots/tabs.png?raw=true "regular")
![regular](https://github.com/paulrouget/servoshell/blob/master/screenshots/regular.png?raw=true "regular")
![dark theme](https://github.com/paulrouget/servoshell/blob/master/screenshots/dark-theme.png?raw=true "dark theme")
![options](https://github.com/paulrouget/servoshell/blob/master/screenshots/options.png?raw=true "options")
![debug](https://github.com/paulrouget/servoshell/blob/master/screenshots/debug.png?raw=true "debug")
