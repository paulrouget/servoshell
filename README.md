# servoshell

This is a sandbox project. Prototyping and experimenting with embedding Servo.

## Build

There are 2 versions of the UI:
1. Full UI: Tabs + urlbar interface. Cocoa based (only MacOS).
2. Mini UI: No visual controls. Driven by keybindings (Windows, Linux, Mac).

The minimal UI can be compiled on MacOS with `--features=force-glutin`.

## Full UI

A regular browser user interface.

![Full UI](https://github.com/paulrouget/servoshell/blob/master/screenshots/tabs.png?raw=true "Full UI")

## Mini UI

Same features as a Full UI, just no widgets. Tabs are displayed in the titlebar as text.

![Mini UI](https://github.com/paulrouget/servoshell/blob/master/screenshots/mini.png?raw=true "Mini UI")

### Linux and Mac

1. ``rustup install `cat rust-toolchain` ``
2. `cargo build --release`
3. `cargo run --release`

### Windows

Make sure you installed all the [dependencies necessary to build Servo](https://github.com/servo/servo#on-windows-msvc).

1. `mach build -r`
2. `mach run -r`

## How to update Servo

1. change `rev` in `Cargo.toml`
2. copy `rust-toolchain` to `servoshell/rust-toolchain`
3. copy `servo/Cargo.lock` to `servoshell/Cargo.lock`
4. copy `servo/resources` to `servoshell/servo_resources`

## Screenshots

![regular](https://github.com/paulrouget/servoshell/blob/master/screenshots/regular.png?raw=true "regular")
![dark theme](https://github.com/paulrouget/servoshell/blob/master/screenshots/dark-theme.png?raw=true "dark theme")
![options](https://github.com/paulrouget/servoshell/blob/master/screenshots/options.png?raw=true "options")
![debug](https://github.com/paulrouget/servoshell/blob/master/screenshots/debug.png?raw=true "debug")
