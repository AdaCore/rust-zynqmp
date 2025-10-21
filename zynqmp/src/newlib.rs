use core::{arch::asm, ffi::c_long, sync::atomic::AtomicUsize};

use embedded_io::Write;

use crate::{soft_reset, uart};

/// Performs a soft reset.
#[unsafe(no_mangle)]
pub extern "C" fn _exit(_status: i32) -> ! {
    soft_reset()
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
        static mut __sheap: u8;
        static __heap_size: u8;
    }

    static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

    if let Ok(nbytes) = usize::try_from(nbytes) {
        let allocated = ALLOCATED.load(core::sync::atomic::Ordering::Relaxed);

        if let Some(new_allocated) = allocated.checked_add(nbytes) {
            if new_allocated <= unsafe { &__heap_size as *const u8 as usize } {
                ALLOCATED.store(new_allocated, core::sync::atomic::Ordering::Relaxed);
                return unsafe { (&raw mut __sheap).add(allocated) };
            }
        }
    };

    core::ptr::null_mut()
}

/// Provides timing information.
///
/// The timing information is measured in seconds. On Linux systems, time is measured in clock
/// ticks and applications use sysconf(_SC_CLK_TCK) to determine the number of clock ticks per
/// second. As Rust's libc binding assumes a default value of 2 for Linux-like systems, we use this
/// value to adjust the returned time accordingly.
#[unsafe(no_mangle)]
pub extern "C" fn times(buf: *mut tms) -> c_long {
    const _SC_CLK_TCK: u64 = 2;
    let Ok(time) = (cntvct() * _SC_CLK_TCK / cntfrq()).try_into() else {
        return -1;
    };
    unsafe {
        (*buf).tms_utime = time;
        (*buf).tms_stime = 0;
        (*buf).tms_cutime = 0;
        (*buf).tms_cstime = 0;
    }
    time
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct tms {
    pub tms_utime: c_long,
    pub tms_stime: c_long,
    pub tms_cutime: c_long,
    pub tms_cstime: c_long,
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
