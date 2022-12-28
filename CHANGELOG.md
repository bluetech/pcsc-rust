# pcsc 2.8.0 (2022-12-29)

* The `pcsc-sys` crate is now reexported from the `pcsc` crate as
  `pcsc::ffi`.

  Contributed by Basix.

* Added `Card::transmit2()`, which is like `Card::transmit()`, but also
  returns the required receive buffer size on `Error::InsufficientBuffer`
  errors.

  Make sure to heed the note on the function's documentation if you intend
  to use it!

  Contributed by Nicolas Dusart.

* Added `ReaderState::current_state() -> State` getter. This returns the
  last current state that was set using `sync_current_state()`.

  Note that for observing the current state as reported by
  `get_status_change()`, you want `event_state()`, not `current_state()`.

  Contributed by Ilya Averyanov.

* `ReaderState` is now `Send` and `Sync`.

* Bumped the minimum supported rust version (MSRV) 1.20 -> 1.38.

# pcsc 2.7.0 (2022-02-15)

* Added `Card::transaction2()`, which is like `Card::transaction()`, but
  also returns the `&mut self` on error. Without it is is difficult
  (impossible) to encapsulate a retry-loop of trying to acquire a
  transaction in a function, at least until some borrow-checker
  improvements end our woes.

  Contributed by Neal H. Walfield.

# pcsc 2.6.0 (2021-08-24)

* Un-deprecated the `pcsc::Status` type. It was previously disabled due to
  a portability hazard; now Windows is made to behave like other
  platforms, although only one bit is ever set.

- Added `status.status()` accessor to `CardStatus` (return value of
  `Card::status2()`) and `CardStatusOwned` (return value of
  `Card::status2_owned()`).

# pcsc 2.5.0 (2021-05-17)

* Added owned versions of several functions:
  - `Context::list_readers` -> `Context::list_readers_owned`
  - `Card::get_attribute` -> `Card::get_attribute_owned`
  - `Card::status2` -> `Card::status2_owned`
  You can use them if you don't want to deal with lengths and buffers and
  just allocate.

  Contributed by Robin Krahl.

# pcsc 2.4.0 (2020-04-20)

* Added support for direct connections to a reader (without a card
  attached).

  This enables setting control parameters on readers which support it, for
  example. Transmitting with custom protocols on such connections is not
  supported however.

  Such connections use `ShareMode::Direct` and `Protocols::UNDEFINED`.

  For this purpose the following changes were made:

  - Added a `CardStatus::protocol2()` which returns `Option<Protocol>`, as
    opposed to `CardStatus::protocol()` which now panics when missing
    protocol.

  - Made `Card::status()` panic when missing protocol. This function is
    deprecated (use `Card::status2()` instead).

  - Made `Card::transmit()` panic when missing protocol.

# pcsc 2.3.1 (2019-12-19)

* Fixed wrong result from `Error::into_raw()` on 64-bit Linux/BSD.

  An incorrect cast caused a bit extension to be made when `LONG` is 64 bit.
  This means 64-bit platforms other than Windows and macOS (where `LONG` is
  always 32 bit).

  Internally, `Error::into_raw()` is used in `Context::list_readers()` and
  `Context::list_readers_len()`, which had a bug as a result: instead of
  returning empty iterator/`Ok(0)` when no readers are available,
  `Err(Error::NoReadersAvailable)` was returned.

# pcsc 2.3.0 (2019-11-25)

* Deprecated the `Card::status()` function and the `Status` type.
  Turns out `Status` is not a bitmask on Windows, so the interface was not
  portable and unfixable.

  These were replaced with `Card::status2()`, which returns a new
  `CardStatus` type. This interface is portable, and also exposes the ATR
  and reader names. However, it doesn't yet expose the actual status -- if
  you need it, please open an issue.

* Implemented `Debug` for the `ReaderNames` iterator struct.

# pcsc-sys 1.2.0 (2019-11-25)

* Added `SCARD_ATR_LENGTH` constant.

* Fixed values of card state constants (`SCARD_PRESENT` and friends) on
  Windows. They were entirely wrong there.

* Fixed value of `SCARD_PROTOCOL_RAW` on Windows.

# pcsc 2.2.0 (2019-09-03)

* Added a `ctl_function()`, a wrapper for the `SCARD_CTL_CODE` PCSC API.

# pcsc-sys 1.1.0 (2019-09-03)

* Added bindings for the `SCARD_CTL_CODE` PCSC API.

* Added a way to specify which PCSC library to use directly instead of
  using pkg-config, if needed. The build script now checks for two
  environment variables, `PCSC_LIB_DIR` and `PCSC_LIB_NAME`. So the crate
  documentation for details.

# pcsc 2.1.1 (2019-02-11)

* Fixed bug where card.disconnect() would keep the associated Context
  alive (leaked a strong reference).

# pcsc 2.1.0 (2019-02-11)

* Added support for getting the ATR (Answer To Reset) from a
  `ReaderState`.

# pcsc 2.0.0 (2018-12-01)

* **BREAKING CHANGE** (`pcsc`) Remove the lifetime from the `Card` type.

  This lifetime made the common case of bundling a `Context` and a `Card`
  in the same struct very difficult, due to Rust's current inability to
  express "self-referential structs".

  Instead of the lifetime, a `Card` now holds an `Arc` reference to its
  `Context`. The consequences of this are:

  - `Context` now implements `Clone`. The implementation is an
    `Arc::clone`.

  - The PCSC context is only released once the last clone is dropped.

  - The function `Context::release()` fails with an `Error::CantDispose`
    if called while there are other clones alive.

  - The "transitive" `'card` lifetime of `Transaction` is removed.

  The migrate your code, replace all instances of `Card<'lifetime>` with
  `Card`, if you had to explicitly specify the lifetime.

  Note that the `'tx` lifetime of `Transaction` remains. Transactions
  are normally short-lived and well-scoped, so using a static lifetime
  makes better sense in this case.

  Reported by @a-dma in
  [issue #9](https://github.com/bluetech/pcsc-rust/issues/9).

# 1.0.1 (2018-02-25)

* **BREAKING CHANGE** (`pcsc-sys`) Fix the types of `SCARDCONTEXT` and
  `SCARDHANDLE` on Windows.

  Previously they were (effectively) defined as `i32`, but the correct
  definition is `usize`. Hence on all Windows platforms, the sign is
  different (not really a problem); and on 64 bits, the width is
  different (wouldn't work).

  While this is technically a breaking change, this should be transparent
  and binaries for Windows 64 bits wouldn't have worked anyway, so we
  avoid a major version bump which is a headache for FFI crates.

  Reported and debugged by @ndusart in
  [issue #6](https://github.com/bluetech/pcsc-rust/issues/6).

* (`pcsc`) Depend on `pcsc-sys >= 1.0.1` to discourage using the broken
  `1.0.0`.


# 1.0.0 (2017-12-05)

* **BREAKING CHANGE** Update bitflags to version 1. This makes the flag
  types easier to use, and improves the documentation.

  Since bitflags now uses associated constants, Rust >= 1.20 is required.

  Example for updating: `STATE_UNAWARE` -> `State::UNAWARE`.

* The `pcsc-sys` crate is also promoted to 1.0.0, without changes.


# 0.1.2 (2017-08-16)

* `pcsc-sys`: Added `SCardControl()` bindings.

* Added `Card::control()`, a wrapper over `SCardControl()`.

* `pcsc-sys`: Improved build target detection in the build script.


# 0.1.1 (2017-06-15)

* Fixed errors in the macOS bindings. In particular, wrong integer types
  were used, and some structs had incorrect padding.

  All discovered problems were fixed; the library now works correctly on
  macOS.

  Reported and debugged by @RokLenarcic in
  [issue #4](https://github.com/bluetech/pcsc-rust/issues/4).

* The function `ReaderState::new()` now takes `Into<CString>` instead of
  `&CStr`. Previously the `&CStr` was turned into a `CString` internally;
  the new form is more explicit and can avoid an allocation if a `CString`
  is passed directly.


# 0.1.0 - 2017-02-06

Initial stable release.
