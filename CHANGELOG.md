# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-05-13

### Added

- `semihosting` feature: enables clean QEMU termination via semihosting `SYS_EXIT` calls, preventing hanging processes when running `cargo test` under QEMU

### Fixed

- `newlib`: `times()` syscall now returns elapsed time in microseconds, improving `std::time::Instant` from 1 s resolution to 1 µs resolution (this is a Rust-internal convention with the standard library's newlib PAL; requires GNAT Pro for Rust 27 or later)

## [0.1.1] - 2026-02-18

### Fixed

- Initialization of `.bss` segment

## [0.1.0] - 2026-02-04

### Added

- Arm Cortex-A53 (AArch64) support
- MMU configuration enabling caching and enforcing memory protection (W^X enforcement, guard pages)
- Custom interrupt handling
- UART driver
- Optional partial `std` support

[0.2.0]: https://github.com/AdaCore/rust-zynqmp/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/AdaCore/rust-zynqmp/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/AdaCore/rust-zynqmp/releases/tag/v0.1.0
