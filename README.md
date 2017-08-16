# servoshell

This is a sandbox project. Prototyping and experimenting with embedding Servo.
The code compiles and run, but it's messy and memory management is non existent.

## Screenshots

![tabs](https://github.com/paulrouget/servoshell/blob/master/screenshots/tabs.png?raw=true "regular")
![regular](https://github.com/paulrouget/servoshell/blob/master/screenshots/regular.png?raw=true "regular")
![dark theme](https://github.com/paulrouget/servoshell/blob/master/screenshots/dark-theme.png?raw=true "dark theme")
![options](https://github.com/paulrouget/servoshell/blob/master/screenshots/options.png?raw=true "options")
![debug](https://github.com/paulrouget/servoshell/blob/master/screenshots/debug.png?raw=true "debug")

## How to update Servo

1. change `rev` in `Cargo.toml`
2. update `rust-toolchain`
3. copy servo/Cargo.lock to servoshell/Cargo.lock
4. copy servo/resources to servoshell/servo_resources
