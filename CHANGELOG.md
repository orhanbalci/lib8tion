# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-08-05

### Added
- `blend8_8bit_full_range` for compatibility with the `color8` crate.

### Changed
- Manifest metadata and `docs.rs` targets (`thumbv7em-none-eabihf`, `riscv32imc-unknown-none-elf`).

## [0.1.0] - 2026-06-07

Initial public release.

### Added
- Rust port of FastLED's `lib8tion` fast 8-bit math primitives for embedded LED programming.
- `no_std` support.
- Integration test suite.
- README.

### Changed
- Rust ergonomics refactoring ahead of publishing.

[Unreleased]: https://github.com/orhanbalci/lib8tion/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/orhanbalci/lib8tion/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/orhanbalci/lib8tion/releases/tag/v0.1.0
