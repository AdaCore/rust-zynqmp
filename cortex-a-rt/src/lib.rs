#![no_std]

use core::arch::global_asm;

use aarch64_cpu::registers::{Writeable, *};

pub mod uart;

global_asm!(include_str!("start.S"));

unsafe extern "C" {
    static __vectors: u64;
}

#[unsafe(no_mangle)]
unsafe fn __set_exception_vector_table_el1() {
    unsafe {
        let vectors_address = &__vectors as *const u64 as u64;
        VBAR_EL1.set(vectors_address);
    }
}

#[unsafe(no_mangle)]
fn __enable_fpu_el1() {
    CPACR_EL1.write(CPACR_EL1::FPEN::SET);
}

#[macro_export]
macro_rules! entry {
    ($path:path) => {
        #[unsafe(no_mangle)]
        pub unsafe fn __main() -> ! {
            let f: fn() -> ! = $path;
            f()
        }
    };
}
