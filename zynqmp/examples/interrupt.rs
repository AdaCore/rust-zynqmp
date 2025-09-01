#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU8, Ordering};

use aarch64_cpu::registers::{CNTV_CTL_EL0, CNTV_TVAL_EL0, Writeable};
use arm_gic::{
    IntId,
    gicv2::{
        GicV2,
        registers::{Gicc, Gicd},
    },
    irq_enable,
};
use qemu_exit::QEMUExit;
use zynqmp::uart::Write;

zynqmp::entry!(main);

static G: AtomicU8 = AtomicU8::new(0);

fn main() -> ! {
    trigger_sync_interrupt();

    assert_eq!(G.load(Ordering::Relaxed), 1);

    trigger_irq_interrupt();

    assert_eq!(G.load(Ordering::Relaxed), 11);

    trigger_sync_interrupt();

    assert_eq!(G.load(Ordering::Relaxed), 12);

    trigger_irq_interrupt();

    assert_eq!(G.load(Ordering::Relaxed), 22);

    qemu_exit::AArch64::new().exit_success();
}

fn trigger_sync_interrupt() {
    unsafe {
        core::arch::asm!("svc #0");
    }
}

#[unsafe(no_mangle)]
extern "C" fn _sync_handler() {
    let l = 1;

    G.store(G.load(Ordering::Relaxed) + l, Ordering::Relaxed);
}

fn trigger_irq_interrupt() {
    const GICD_BASE_ADDRESS: *mut Gicd = 0xf901_0000u32 as _;
    const GICC_BASE_ADDRESS: *mut Gicc = 0xf902_0000u32 as _;

    unsafe {
        // Enable GIC distributor
        (GICD_BASE_ADDRESS as *mut u8).write_volatile(1);
        // Enable GIC CPU interface
        (GICC_BASE_ADDRESS as *mut u8).write_volatile(1);
    }

    // SAFETY: `GICD_BASE_ADDRESS` and `GICC_BASE_ADDRESS` are the base addresses of a GICv2
    // distributor and CPU interface respectively, and nothing else accesses those address ranges.
    let mut gic = unsafe { GicV2::new(GICD_BASE_ADDRESS, GICC_BASE_ADDRESS) };
    let timer_irq = IntId::ppi(11);

    gic.set_priority_mask(0xFF);
    gic.set_interrupt_priority(timer_irq, 0x80);
    gic.enable_interrupt(timer_irq, true).unwrap();
    irq_enable();

    // Enable timer
    CNTV_TVAL_EL0.set(1);
    CNTV_CTL_EL0.write(CNTV_CTL_EL0::ENABLE::SET);
}

#[unsafe(no_mangle)]
extern "C" fn _irq_handler() {
    // Disable timer
    CNTV_CTL_EL0.write(CNTV_CTL_EL0::ENABLE::CLEAR);

    let l = 10;

    G.store(G.load(Ordering::Relaxed) + l, Ordering::Relaxed);
}

#[panic_handler]
fn panic(panic: &core::panic::PanicInfo<'_>) -> ! {
    let mut uart = unsafe { zynqmp::uart::uart0() };
    uart.initialize();
    writeln!(uart, "Panic: {}", panic.message()).unwrap();
    qemu_exit::AArch64::new().exit_failure();
}
