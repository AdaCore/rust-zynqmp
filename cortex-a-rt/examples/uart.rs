#![no_std]
#![no_main]

use embedded_io::Write;
use qemu_exit::QEMUExit;

cortex_a_rt::entry!(main);

fn main() -> ! {
    // SAFETY: Nobody else is going to access UART0.
    let mut uart = unsafe { cortex_a_rt::uart::uart0() };
    writeln!(uart, "Hello world, the answer is {}!", 42).unwrap();
    panic!("{} went (intentionally) wrong", "Something");
}

#[panic_handler]
fn panic(panic: &core::panic::PanicInfo<'_>) -> ! {
    // SAFETY: It's possible that other parts of the code are using the UART,
    // but we're panicking, so let's try to get a final message out.
    let mut uart = unsafe { cortex_a_rt::uart::uart0() };
    writeln!(uart, "Panic: {}", panic.message()).unwrap();

    // We actually expect a panic in this program, so let's exit with a success
    // code to make testing easier.
    qemu_exit::AArch64::new().exit_success()
}
