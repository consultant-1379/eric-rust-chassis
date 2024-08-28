# Developer Manual

[TOC]

## General Rust tips and tools

### Shared build cache

Install `sscache` via Cargo and set the following environment variable to use it.

While it can speed up compilations across workspaces, the built artifacts are
not shared, therefore disk space cannot be saved.

Be aware of [caveats](https://github.com/mozilla/sccache?tab=readme-ov-file#known-caveats).

```sh
cargo install sccache
export RUSTC_WRAPPER=sccache
```

Reference: https://doc.rust-lang.org/cargo/guide/build-cache.html#shared-cache

### Coverage report

```sh
cargo install cargo-llvm-cov
cargo llvm-cov --html
```

### 3PP handling

Licenses can be easily collected via `cargo license`. Install it and use it
with the following commands.

```sh
cargo install cargo-license
cargo license
```
