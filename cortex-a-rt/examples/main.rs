#![no_std]
#![no_main]

extern crate cortex_a_rt;

cortex_a_rt::entry!(main);

static RODATA: &[u8] = b"Hello, world!";
static mut BSS: u8 = 0;
static mut DATA: u16 = 1;

fn main() -> ! {
    let _x = RODATA;
    let _y = &raw const BSS;
    let _z = &raw const DATA;

    #[allow(clippy::empty_loop)]
    loop {}
}

#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}
