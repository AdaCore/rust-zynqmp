#![allow(clippy::needless_doctest_main)]
#![warn(missing_docs)]
#![warn(rustdoc::private_doc_tests)]
#![warn(rustdoc::unescaped_backticks)]
//! Support for the AMD Zynq UltraScale+ MPSoC.
//!
//! # Usage
//!
//! The crate expects a file called `memory.x`, which defines the memory map for a specific board,
//! in the package root of your Cargo project. Here is a sample memory map for the ZCU102 board:
//!
//! ```text
//! MEMORY
//! {
//!   DDR (rwx) : ORIGIN = 0, LENGTH = 2048M
//!   OCM (rwx) : ORIGIN = 0xFFFC0000, LENGTH = 256K
//! }
//!
//! REGION_ALIAS("CODE", DDR)
//! REGION_ALIAS("DATA", DDR)
//! ```
//!
//! Cargo will generate a linker script called `link.x` when the `zynqmp` crate is built. You must
//! ensure that this linker script is used by adding the following flag to your
//! `.cargo/config.toml`:
//!
//! ```text
//! [target.aarch64-unknown-none]
//! rustflags = [
//!     "-C", "link-arg=-Tlink.x"
//! ]
//! ```
//!
//! Alternatively, this flag can be defined in a build script or the `RUSTFLAGS` environment
//! variable.
//!
//! The [`std`](#std) feature enables limited support for `std`. The `zynqmp` crate must be
//! referenced in a use import to ensure that the crate is linked into the binary:
//!
//! ```
//! use zynqmp as _;
//!
//! fn main() {
//!     println!("Hello, world!");
//! }
//! ```
//!
//! In a `no_std` project the entry point must be defined using the [`entry`] macro.
//!
//! ## Interrupt Handling
//!
//! You can define custom interrupt handling by implementing the following functions:
//!
//! ```
//! #[unsafe(no_mangle)]
//! extern "C" fn _sync_handler() {
//!     // ...
//! }
//!
//! #[unsafe(no_mangle)]
//! extern "C" fn _irq_handler() {
//!     // ...
//! }
//!
//! #[unsafe(no_mangle)]
//! extern "C" fn _fiq_handler() {
//!     // ...
//! }
//!
//! #[unsafe(no_mangle)]
//! extern "C" fn _serror_handler() {
//!     // ...
//! }
//! ```
//!
//! By default, an interrupt leads to a spin loop.
//!
//! ## Exit Handling
//!
//! By default, a soft reset will be performed if an exit point is reached. You can change this
//! behavior by defining a custom exit handler:
//!
//! ```
//! #[unsafe(no_mangle)]
//! extern "C" fn _exit_handler() {
//!     // ...
//! }
//! ```
//!
//! # Optional Features
//!
//! ## `std`
//!
//! Provides limited support for `std` based on [Newlib](https://www.sourceware.org/newlib/).
//! Requires GNAT Pro for Rust 26 or newer.
//!
//! # Minimum Supported Rust Version (MSRV)
//!
//! This crate is guaranteed to compile on stable Rust 1.85.0 and up. It might compile with older
//! versions but that may change in any new patch release.

#![no_std]

use core::arch::global_asm;

use aarch64_cpu::registers::{CPACR_EL1, VBAR_EL1, Writeable};

mod crl_apb;
mod mmu;
#[cfg(feature = "std")]
mod newlib;
pub mod uart;

global_asm!(include_str!("vectors.S"));
global_asm!(include_str!("start.S"));

unsafe extern "C" {
    static __vectors: u64;
    fn main();
    fn _exit_handler() -> !;
}

/// Entry point into Rust code.
///
/// Performs some more hardware setup and then calls `main`, the entry point
/// into user code (see `entry` below).
#[unsafe(no_mangle)]
extern "C" fn __start_rust() -> ! {
    set_exception_vector_table_el1();
    enable_fpu_el1();
    mmu::enable();
    unsafe {
        main();
        _exit_handler();
    }
}

fn set_exception_vector_table_el1() {
    unsafe {
        let vectors_address = &__vectors as *const u64 as u64;
        VBAR_EL1.set(vectors_address);
    }
}

fn enable_fpu_el1() {
    CPACR_EL1.write(CPACR_EL1::FPEN::SET);
}

/// Performs a soft reset.
pub fn soft_reset() -> ! {
    let mut crl_apb = crl_apb::crl_apb();

    crl_apb.modify_crl_wprot(|crl_wprot| crl_wprot.with_active(false));

    loop {
        crl_apb.modify_reset_ctrl(|reset_ctrl| reset_ctrl.with_soft_reset(true));
    }
}

/// Performs a soft reset.
///
/// Executed if an exit point is reached and the exit handler has not been overridden.
#[unsafe(no_mangle)]
extern "C" fn __default_exit_handler() {
    soft_reset()
}

/// Executes a busy-wait spin-loop.
///
/// Executed if an exception occurs and the specific exception handler has not been overridden.
#[unsafe(no_mangle)]
extern "C" fn __default_handler() {
    loop {
        core::hint::spin_loop();
    }
}

/// Defines the entry point of the binary.
///
/// # Example
///
/// ```
/// #![no_std]
/// #![no_main]
///
/// use zynqmp::{entry, soft_reset};
///
/// entry!(main);
///
/// fn main() {}
///
/// #[panic_handler]
/// fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
///     soft_reset();
/// }
/// ```
#[macro_export]
macro_rules! entry {
    ($path:path) => {
        #[unsafe(export_name = "main")]
        pub extern "C" fn __main() {
            $path()
        }
    };
}
