# pcsc-rust

[![Travis CI](https://travis-ci.org/bluetech/pcsc-rust.svg?branch=master)](https://travis-ci.org/bluetech/pcsc-rust)
[![AppVeyor](https://ci.appveyor.com/api/projects/status/s16sb4kt79v7yop4/branch/master?svg=true)](https://ci.appveyor.com/project/bluetech/pcsc-rust/branch/master)
[![crates.io](https://img.shields.io/crates/v/pcsc.svg?maxAge=2592000)](https://crates.io/crates/pcsc)
[![docs.rs](https://docs.rs/pcsc/badge.svg)](https://docs.rs/pcsc)

Rust bindings to PC/SC for smart card communication.

- Nice, safe API.
- Works on all major operating systems.
- Mostly zero overhead.

See the [Documentation](https://docs.rs/pcsc) for more details.

See the examples directory for some common tasks.

## Usage

In your `Cargo.toml`:

```toml
[dependencies]
pcsc = "0.1"
```

In your crate:

```rust
extern crate pcsc;
```

## Status

The library is perfectly usable, however it is still using a pre-release
version because:

- There are still some [TODOs](https://github.com/bluetech/pcsc-rust/search?q=TODO)
  remaining.

- Apple support is only compile-tested.

Help is welcome!

## License

The MIT license, see the LICENSE file.
