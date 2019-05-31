# Dargo

[![release](https://img.shields.io/crates/v/dargo.svg)](https://crates.io/crates/dargo)
[![dependency status](https://deps.rs/repo/github/dcjanus/dargo/status.svg)](https://deps.rs/repo/github/dcjanus/dargo)
[![LOC](https://tokei.rs/b1/github/dcjanus/dargo)](https://github.com/dcjanus/dargo)

Some useful third-party tools based [Cargo](https://github.com/rust-lang/cargo).

# Install

Latest released: `cargo install -f dargo`

Latest branched: `cargo install -f dargo --git https://github.com/DCjanus/dargo.git`

# Commands

## dargo upgrade

Upgrade version requirements in `Cargo.toml` to latest, more usage: `dargo upgrade -h`.

Version requirements like `1.2.3`, `0.1.2`, always updated to the latest version. Version requirements like `^1.2.3`, `~2.0.1`, `=0.3.0`, `1.2`, `1.*`, updated when latest version does not meet current constraints.

PS: Which means, `>`, `>=`, `*` will never be updated.

## dargo add

Add dependencies to your `Cargo.toml`, more usage: `dargo add -h`.

Without version specified like `dargo add failure libc`, add latest version dependencies to `Cargo.toml`.

With version requirement like `dargo add futures-preview@>=0.3.0-alpha.12 libc@^1.0.1`, add dependencies with specified version requirement.

## dargo rm

Remove dependencies from your `Cargo.toml`, more usage: `dargo rm -h`.

# Tips

+ `dargo upgrade` and `dargo add` would not update local registry index, unless run with flag `--update`.

# Position

There are some useful CLI tools for Rust developers, for example, [cargo-edit](https://github.com/killercup/cargo-edit), [cargo-outdated](https://github.com/kbknapp/cargo-outdated).

They are fabulous, but for some reason, some designs are not suitable for my needs, for example: for compile speed, `cargo-edit` doesn't rely on [cargo](https://crates.io/crates/cargo), and query latest version of crate via HTTP API, which is so slow in China. So I have to write `dargo` for myself and query latest version from local registry index.

# TODO 

## dargo outdated

Check dependencies in Cargo.lock
