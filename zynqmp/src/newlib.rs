//! # Newlib Support

use core::{arch::asm, ffi::c_long, sync::atomic::AtomicUsize};

use embedded_io::Write;

#[cfg(not(feature = "semihosting"))]
use crate::soft_reset;
use crate::uart;

/// Terminates execution.
///
/// With the `semihosting` feature, uses a semihosting `SYS_EXIT` call so that
/// QEMU exits cleanly. Without it, performs a soft reset.
#[unsafe(no_mangle)]
pub extern "C" fn _exit(status: i32) -> ! {
    #[cfg(feature = "semihosting")]
    {
        const SYS_EXIT: usize = 0x18;
        const ADP_STOPPED_APPLICATION_EXIT: usize = 0x20026;
        const ADP_STOPPED_RUN_TIME_ERROR: usize = 0x20023;

        let reason = if status == 0 {
            ADP_STOPPED_APPLICATION_EXIT
        } else {
            ADP_STOPPED_RUN_TIME_ERROR
        };

        unsafe {
            asm!(
                "hlt #0xf000",
                in("x0") SYS_EXIT,
                in("x1") reason,
                options(noreturn)
            );
        }
    }
    #[cfg(not(feature = "semihosting"))]
    {
        let _ = status;
        soft_reset()
    }
}

/// Performs a soft reset.
#[unsafe(no_mangle)]
pub extern "C" fn abort() -> ! {
    _exit(-1)
}

/// Writes the given data to UART0 and returns the number of written bytes.
///
/// # Panics
///
/// This function will panic if writing to UART0 fails.
#[unsafe(no_mangle)]
pub extern "C" fn write(_fd: i32, buf: *const u8, nbytes: i32) -> i32 {
    let mut uart = unsafe { uart::uart0() };

    if let Ok(len) = usize::try_from(nbytes) {
        if uart
            .write_all(unsafe { core::slice::from_raw_parts(buf, len) })
            .is_ok()
        {
            nbytes
        } else {
            0
        }
    } else {
        0
    }
}

/// Allocates memory and returns a pointer to the start of the allocated region.
///
/// # Safety
///
/// This function is **not thread-safe**. It modifies a global allocator state without any
/// synchronization.
#[unsafe(no_mangle)]
pub extern "C" fn sbrk(nbytes: i32) -> *mut u8 {
    unsafe extern "C" {
        static mut __heap_start: u8;
        static __heap_size: u8;
    }

    static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

    if let Ok(nbytes) = usize::try_from(nbytes) {
        let allocated = ALLOCATED.load(core::sync::atomic::Ordering::Relaxed);

        if let Some(new_allocated) = allocated.checked_add(nbytes) {
            if new_allocated <= unsafe { &__heap_size as *const u8 as usize } {
                ALLOCATED.store(new_allocated, core::sync::atomic::Ordering::Relaxed);
                return unsafe { (&raw mut __heap_start).add(allocated) };
            }
        }
    };

    core::ptr::null_mut()
}

/// Provides timing information.
///
/// The value written to `tms_utime` (and returned) is the elapsed time in
/// microseconds. This is a convention shared with the Rust standard library's
/// newlib PAL, whose `Instant::now()` interprets `clock()` as microseconds via
/// `Duration::from_micros(clk)`. Newlib's `clock()` returns the sum of the
/// four `tms` fields without scaling, so by zeroing the other three fields
/// here the microsecond value flows through unchanged.
///
/// This convention is independent of newlib's `CLOCKS_PER_SEC` macro, which
/// may differ from 1_000_000 on this target. The Rust call chain
/// (`Instant::now()` -> `clock()` -> `times()`) is internally consistent at
/// microsecond resolution, but C code that calls `clock()` and divides by
/// `CLOCKS_PER_SEC` to obtain seconds will compute the wrong result whenever
/// the macro disagrees with the microsecond convention. This crate is not
/// currently usable for accurate POSIX-conforming timing from C.
#[unsafe(no_mangle)]
extern "C" fn times(buf: *mut tms) -> c_long {
    const MICROS_PER_SEC: u128 = 1_000_000;

    // u128 avoids overflow in the intermediate product.
    let ticks = (cntvct() as u128) * MICROS_PER_SEC / (cntfrq() as u128);
    let Ok(ticks) = c_long::try_from(ticks) else {
        return -1;
    };
    unsafe {
        (*buf).tms_utime = ticks;
        (*buf).tms_stime = 0;
        (*buf).tms_cutime = 0;
        (*buf).tms_cstime = 0;
    }
    ticks
}

#[repr(C)]
#[allow(non_camel_case_types)]
struct tms {
    tms_utime: c_long,
    tms_stime: c_long,
    tms_cutime: c_long,
    tms_cstime: c_long,
}

#[inline]
fn cntvct() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {0}, cntvct_el0", out(reg) value);
    }
    value
}

#[inline]
fn cntfrq() -> u64 {
    let freq: u64;
    unsafe {
        asm!("mrs {0}, cntfrq_el0", out(reg) freq);
    }
    freq
}
