# rupsc

[![crates.io](https://img.shields.io/crates/v/rupsc.svg)](https://crates.io/crates/rupsc)
[![Documentation](https://docs.rs/nut-client/badge.svg)](https://docs.rs/nut-client)
[![MIT licensed](https://img.shields.io/crates/l/rupsc.svg)](./LICENSE)
[![CI](https://github.com/aramperes/nut-client-rs/workflows/CI/badge.svg)](https://github.com/aramperes/nut-client-rs/actions?query=workflow%3ACI)

A Rust clone of [upsc](https://networkupstools.org/docs/man/upsc.html),
the [Network UPS Tools](https://github.com/networkupstools/nut) (NUT) demo program to display UPS variables.

Written using the [nut-client](https://github.com/aramperes/nut-client-rs) crate.

- Connect to `upsd`/`nut-server` using TCP
- List UPS devices
- List variables for a UPS device
- Get variable value of a UPS device
- List clients connected to a UPS device
- Connect securely with SSL

## Installation

```bash
# Using cargo
cargo install rupsc

# Or, build for other targets
# (make sure you install the appropriate toolchain & gcc linker)
cargo build --release --target armv7-unknown-linux-gnueabihf
cargo build --release --target aarch64-unknown-linux-gnu
cargo build --release --target arm-unknown-linux-gnueabihf
```

## Usage

This is a clone of [`upsc`](https://networkupstools.org/docs/man/upsc.html), so the usage is the same:

```bash
# Show usage
rupsc -h

# List variables on UPS device "nutdev1" (assumes upsd running on 127.0.0.1:3493)
rupsc nutdev1

# List variables on UPS device "nutdev1" (remove upsd)
rupsc nutdev1@192.168.1.2:3493

# List available UPS devices
rupsc -l

# List available UPS devices, with description
rupsc -L

# List clients connected to UPS device "nutdev1"
rupsc -c nutdev1
```

However, there are also some additions:

```bash
# Enable network debugging (global flag).
ruspc -D

# Enable SSL
rupsc -S
```

## Pronunciation

> r-oopsie
