use core::sync::atomic::AtomicUsize;

use embedded_io::Write;

use crate::{crl_apb, uart};

/// Performs a soft reset.
#[unsafe(no_mangle)]
pub extern "C" fn _exit(_status: i32) -> ! {
    let mut crl_apb = crl_apb::crl_apb();

    crl_apb.modify_crl_wprot(|crl_wprot| crl_wprot.with_active(false));

    loop {
        crl_apb.modify_reset_ctrl(|reset_ctrl| reset_ctrl.with_soft_reset(true));
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
        static mut __sheap: u8;
        static __heap_size: usize;
    }

    static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

    if let Ok(nbytes) = usize::try_from(nbytes) {
        let allocated = ALLOCATED.load(core::sync::atomic::Ordering::Relaxed);

        if let Some(new_allocated) = allocated.checked_add(nbytes) {
            if new_allocated <= unsafe { __heap_size } {
                ALLOCATED.store(new_allocated, core::sync::atomic::Ordering::Relaxed);
                return unsafe { (&raw mut __sheap).add(allocated) };
            }
        }
    };

    core::ptr::null_mut()
}
