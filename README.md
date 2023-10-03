# What is this?

---

A very small example of running a llm within bevy.

---

# NOTES

- You'll need to download your own `RWKV-4-World-0.4B-v1-20230529-ctx4096.st` and place it into assets/models/ since it's too big to put onto github. You can easily get your own from [HERE](https://github.com/cryscan/web-rwkv/blob/main/assets/models/RWKV-4-World-0.4B-v1-20230529-ctx4096.st). The program expects this exact model to be used.

--

# Dependencies

Add the needed targets via `rustup`, currently supported are `x86_64-pc-windows-msvc`, `x86_64-pc-windows-gnu`,`x86_64-unknown-linux-gnu`, and `wasm32-unknown-unknown` (for web).

By default, the `mold` linker will be needed for building for the `x86_64-unknown-linux-gnu` target. Otherwise it can be changed to just lld and thus lld will be needed.

> Directions for installing the needed dependencies for these targets can be found in the bevy cheatbook.

# Commands

The commands to build and run for the supported targets are in `.cargo/config.toml`.

---
