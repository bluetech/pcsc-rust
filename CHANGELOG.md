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
