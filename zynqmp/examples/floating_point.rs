#![no_std]
#![no_main]

use qemu_exit::QEMUExit;

zynqmp::entry!(main);

fn main() -> ! {
    let mut x = 1.0;
    x += 2.0;
    assert_eq!(x, 3.0);

    qemu_exit::AArch64::new().exit_success()
}

#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    qemu_exit::AArch64::new().exit_failure()
}
