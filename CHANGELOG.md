# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-11-24

### Added

- `Encoding::len_char` and `Encoding::fits`.

### Changed

- **Breaking:** `Encoding` implementors must provide `len_char`.
- `BString::try_push`, `BString::try_insert`, and `BString::remove` are faster.
- `BStr::from_str` no longer measures the whole string when its byte length
  already decides the result.

### Removed

- **Breaking:** `LengthExceeded::len`. The error message no longer includes the
  length.

### Fixed

- `BString::retain` keeps `encoded_len` and `remaining` exact if the predicate
  panics.

## [0.2.1] - 2026-11-24

### Added

- `BString::encoded_len` and `BString::remaining`.

## [0.2.0] - 2026-11-23

### Added

- `BStr::get`, `BStr::split_at`, `BStr::split_at_checked`, and `Index` for `BStr`.
- `BStr::trim`, `BStr::trim_start`, and `BStr::trim_end`.
- `PartialEq` between `BStr`, `BString`, `str`, and `String`.

## [0.1.0] - 2026-11-23

### Added

- `BStr` and `BString`.
- `Encoding` trait with `Utf8` and `Utf16`.
- Optional `serde` feature.

[0.3.0]: https://github.com/Byunk/boundstr/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/Byunk/boundstr/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/Byunk/boundstr/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Byunk/boundstr/releases/tag/v0.1.0
