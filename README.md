# What is this?

This is a minimalistic template meant for making games via bevy on wsl2. It is also set up to export to various platforms such as windows, linux, and web

---

# Dependencies

Yes - this template has a few dependencies. Since the aim of this is to be as minimalistic as possible all dependencies will have to be manually downloaded

Add the needed targets via `rustup`, currently supported are `x86_64-pc-windows-msvc`, `x86_64-pc-windows-gnu`,`x86_64-unknown-linux-gnu`, and `wasm32-unknown-unknown` (for web).

By default, the `mold` linker will be needed for building for the `x86_64-unknown-linux-gnu` target. Otherwise it can be changed to just lld and thus lld will be needed.

> Directions for installing the needed dependencies for these targets can be found in the bevy cheatbook.

# Commands

The commands to build and run for the supported targets are in `.cargo/config.toml`.

---
