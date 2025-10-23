#![no_std]

use core::arch::global_asm;

use aarch64_cpu::registers::{CPACR_EL1, VBAR_EL1, Writeable};

mod crl_apb;
mod mmu;
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
fn soft_reset() -> ! {
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
pub extern "C" fn __default_exit_handler() {
    soft_reset()
}

/// Executes a busy-wait spin-loop.
///
/// Executed if an exception occurs and the specific exception handler has not been overridden.
#[unsafe(no_mangle)]
pub extern "C" fn __default_handler() {
    loop {
        core::hint::spin_loop();
    }
}

#[macro_export]
macro_rules! entry {
    ($path:path) => {
        #[unsafe(export_name = "main")]
        pub extern "C" fn __main() {
            $path()
        }
    };
}
