[![crates.io](https://img.shields.io/crates/v/adacore-zynqmp)](https://crates.io/crates/adacore-zynqmp)
[![docs.rs](https://img.shields.io/docsrs/adacore-zynqmp)](https://docs.rs/adacore-zynqmp)

# Support for the AMD Zynq UltraScale+ MPSoC

A Rust crate providing hardware and runtime support for the AMD Zynq UltraScale+ MPSoC platform, targeting Arm Cortex-A53 cores in AArch64 mode.

## Features

- Arm Cortex-A53 (AArch64) support
- MMU configuration enabling caching and enforcing memory protection (W^X enforcement, guard pages)
- Custom interrupt handling
- UART driver
- Optional partial `std` support

## License

This work is licensed under `Apache-2.0`.
