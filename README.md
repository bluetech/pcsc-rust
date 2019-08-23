# pcsc-rust

[![crates.io](https://img.shields.io/crates/v/pcsc.svg)](https://crates.io/crates/pcsc)
[![docs.rs](https://docs.rs/pcsc/badge.svg)](https://docs.rs/pcsc)
[![license](http://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/bluetech/pcsc-rust/blob/master/LICENSE-MIT)

Linux, macOS: [![Travis CI](https://travis-ci.org/bluetech/pcsc-rust.svg?branch=master)](https://travis-ci.org/bluetech/pcsc-rust)
Windows: [![AppVeyor](https://ci.appveyor.com/api/projects/status/s16sb4kt79v7yop4/branch/master?svg=true)](https://ci.appveyor.com/project/bluetech/pcsc-rust/branch/master)

Rust bindings to the PC/SC API for smart card communication.

- Nice, safe API.
- Tested on Linux, Windows, macOS.
- Mostly zero overhead.

See the [Documentation](https://docs.rs/pcsc) for more details.

See the `pcsc/examples` directory for some common tasks.

## Contents

The [`pcsc-sys`](https://docs.rs/pcsc-sys) crate contains direct,
low-level bindings to the C API.

The [`pcsc`](https://docs.rs/pcsc) crate contains high-level Rust
wrappers.

## Usage

In your `Cargo.toml`:

```toml
[dependencies]
pcsc = "2"
```

## Example

Connect to the card in the first available reader, send the card an
APDU command, print the APDU response.

```rust
use pcsc::*;

fn main() {
    // Establish a PC/SC context.
    let ctx = match Context::establish(Scope::User) {
        Ok(ctx) => ctx,
        Err(err) => {
            eprintln!("Failed to establish context: {}", err);
            std::process::exit(1);
        }
    };

    // List available readers.
    let mut readers_buf = [0; 2048];
    let mut readers = match ctx.list_readers(&mut readers_buf) {
        Ok(readers) => readers,
        Err(err) => {
            eprintln!("Failed to list readers: {}", err);
            std::process::exit(1);
        }
    };

    // Use the first reader.
    let reader = match readers.next() {
        Some(reader) => reader,
        None => {
            println!("No readers are connected.");
            return;
        }
    };
    println!("Using reader: {:?}", reader);

    // Connect to the card.
    let card = match ctx.connect(reader, ShareMode::Shared, Protocols::ANY) {
        Ok(card) => card,
        Err(Error::NoSmartcard) => {
            println!("A smartcard is not present in the reader.");
            return;
        }
        Err(err) => {
            eprintln!("Failed to connect to card: {}", err);
            std::process::exit(1);
        }
    };

    // Send an APDU command.
    let apdu = b"\x00\xa4\x04\x00\x0A\xA0\x00\x00\x00\x62\x03\x01\x0C\x06\x01";
    println!("Sending APDU: {:?}", apdu);
    let mut rapdu_buf = [0; MAX_BUFFER_SIZE];
    let rapdu = match card.transmit(apdu, &mut rapdu_buf) {
        Ok(rapdu) => rapdu,
        Err(err) => {
            eprintln!("Failed to transmit APDU command to card: {}", err);
            std::process::exit(1);
        }
    };
    println!("APDU response: {:?}", rapdu);
}
```

Example output:

```
$ ./target/debug/examples/readme
Using reader: "SCM Microsystems Inc. SCR 355 [CCID Interface] 00 00"
Sending APDU: [0, 164, 4, 0, 10, 160, 0, 0, 0, 98, 3, 1, 12, 6, 1]
APDU response: [106, 130]
```

## License

The MIT license.
