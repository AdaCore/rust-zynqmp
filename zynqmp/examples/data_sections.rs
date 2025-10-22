#![no_std]
#![no_main]

use qemu_exit::QEMUExit;

zynqmp::entry!(entry);

static RODATA: &[u8] = b"Hello, world!";
static mut BSS: u8 = 0;
static mut DATA: u16 = 1;

fn entry() -> ! {
    let x = RODATA;
    let y = &raw const BSS;
    let z = &raw const DATA;

    unsafe {
        assert_eq!(x, b"Hello, world!");
        assert_eq!(*y, 0);
        assert_eq!(*z, 1);
    }

    qemu_exit::AArch64::new().exit_success()
}

#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    qemu_exit::AArch64::new().exit_failure()
}
