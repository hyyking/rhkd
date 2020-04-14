# RHKD

Rust X11 Hotkey Daemon (builds on nightly as for )

This package is meant to be an alternative to [sxhkd](https://github.com/baskerville/sxhkd).

# Installation and configuration
1. clone the repository
2. modify the key bindings and modmap in `src/binds.rs`
3. install/run with cargo (resp. `cargo install --path .` | `cargo run`)
