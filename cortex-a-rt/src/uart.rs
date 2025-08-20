// Based on the Zynq UltraScale+ Devices Register Reference (UG1087)
// (https://docs.amd.com/r/en-US/ug1087-zynq-ultrascale-registers/UART-Module)
// and the Zynq UltraScale+ Device Technical Reference Manual (UG1085)
// (https://docs.amd.com/v/u/en-US/ug1085-zynq-ultrascale-trm).

use bitbybit::{bitenum, bitfield};
use embedded_io::{ErrorType, Read, ReadReady, Write, WriteReady};

/// Representation of a UART device.
///
/// The driver uses memory-mapped I/O to control the device, with the address
/// provided on construction.
#[derive(derive_mmio::Mmio)]
#[repr(C)]
struct Uart {
    control: ControlRegister,
    mode: ModeRegister,
    interrupt_enable: u32,
    #[mmio(Write)]
    interrupt_disable: u32,
    interrupt_mask: u32,
    channel_interrupt_status: u32,
    baud_rate_generator: u32,
    receiver_timeout: u32,
    receiver_fifo_trigger_level: u32,
    modem_control: u32,
    modem_status: u32,
    #[mmio(Read)]
    channel_status: ChannelStatusRegister,
    #[mmio(Read, Write)]
    tx_rx_fifo: FifoRegister,
    baud_rate_divider: u32,
    flow_delay: u32,
    _reserved: [u32; 2],
    tx_fifo_trigger_level: u32,
    rx_fifo_byte_status: u32,
}

#[bitfield(u32)]
struct ControlRegister {
    #[bit(5, rw)]
    transmit_disable: bool,
    #[bit(4, rw)]
    transmit_enable: bool,
    #[bit(3, rw)]
    receive_disable: bool,
    #[bit(2, rw)]
    receive_enable: bool,
    #[bit(1, rw)]
    transmit_reset: bool,
    #[bit(0, rw)]
    receive_reset: bool,
}

#[bitfield(u32)]
struct ModeRegister {
    #[bits(8..=9, rw)]
    channel_mode: ChannelMode,
    #[bits(6..=7, rw)]
    stop_bits: Option<StopBits>,
    #[bits(3..=5, rw)]
    parity: Option<Parity>,
    #[bits(1..=2, rw)]
    character_length: Option<CharacterLength>,
    #[bit(0, rw)]
    scale_clock: bool,
}

#[bitenum(u2, exhaustive = true)]
#[allow(dead_code)]
enum ChannelMode {
    Normal = 0b00,
    AutomaticEcho = 0b01,
    LocalLoopback = 0b10,
    RemoteLoopback = 0b11,
}

#[bitenum(u2, exhaustive = false)]
#[allow(dead_code)]
enum StopBits {
    One = 0b00,
    OnePointFive = 0b01,
    Two = 0b10,
}

#[bitenum(u3, exhaustive = false)]
#[allow(dead_code)]
enum Parity {
    Even = 0b000,
    Odd = 0b001,
    ForceZero = 0b010,
    ForceOne = 0b011,
    None = 0b100,
}

#[bitenum(u2, exhaustive = false)]
#[allow(dead_code)]
enum CharacterLength {
    Eight = 0b00,
    Seven = 0b10,
    Six = 0b11,
}

#[bitfield(u32)]
struct ChannelStatusRegister {
    #[bit(11, r)]
    tx_active: bool,
    #[bit(4, r)]
    tx_fifo_full: bool,
    #[bit(3, r)]
    tx_fifo_empty: bool,
    #[bit(1, r)]
    rx_fifo_empty: bool,
}

#[bitfield(u32, default = 0)]
struct FifoRegister {
    #[bits(0..=7, rw)]
    data: u8,
}

/// Obtains a reference to the UART0 device.
///
/// # Safety
///
/// The caller must ensure that no more than a single instance of the returned
/// value exists at any time.
pub unsafe fn uart0() -> MmioUart<'static> {
    // SAFETY: The caller guarantees that nobody else is accessing the device
    // concurrently.
    let mut uart = unsafe { Uart::new_mmio_at(0xff00_0000) };
    uart.initialize();
    uart
}

/// Obtains a reference to the UART1 device.
///
/// # Safety
///
/// The caller must ensure that no more than a single instance of the returned
/// value exists at any time.
pub unsafe fn uart1() -> MmioUart<'static> {
    // SAFETY: The caller guarantees that nobody else is accessing the device
    // concurrently.
    let mut uart = unsafe { Uart::new_mmio_at(0xff01_0000) };
    uart.initialize();
    uart
}

impl MmioUart<'_> {
    /// Initializes the device according to Table 21-5 of the TRM.
    fn initialize(&mut self) {
        // Reset the device and wait for the reset to complete.
        self.modify_control(|control| control.with_transmit_reset(true).with_receive_reset(true));
        loop {
            let control = self.read_control();
            if !control.transmit_reset() && !control.receive_reset() {
                break;
            }
        }

        // Enable the device by clearing the "disable" bits, but leave the
        // "enable" bits off for now.
        self.modify_control(|control| {
            control
                .with_transmit_disable(false)
                .with_receive_disable(false)
                .with_transmit_enable(false)
                .with_receive_enable(false)
        });

        // Now enable transmit and receive.
        self.modify_control(|control| control.with_transmit_enable(true).with_receive_enable(true));

        // Configure normal mode, 8N1.
        self.modify_mode(|mode| {
            mode.with_channel_mode(ChannelMode::Normal)
                .with_character_length(CharacterLength::Eight)
                .with_parity(Parity::None)
                .with_stop_bits(StopBits::One)
        });

        // Disable all interrupts; we use the device synchronously only.
        self.write_interrupt_disable(0x1ff);
    }
}

impl ErrorType for MmioUart<'_> {
    type Error = core::convert::Infallible;
}

impl ReadReady for MmioUart<'_> {
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.read_channel_status().rx_fifo_empty())
    }
}

impl Read for MmioUart<'_> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        // The implementation corresponds to Table 21-9 in the TRM, simplified
        // because we don't use interrupts.

        for (count, byte) in buf.iter_mut().enumerate() {
            if !self.read_ready()? {
                // The contract for this function says that we must not block
                // if `ReadReady::read_ready` has returned true, so instead we
                // have to stop receiving if the FIFO is empty after the first
                // byte. In contrast, we need to block when we can't read the
                // first byte because the contract requires us to block until
                // we can receive at least one byte.
                if count > 0 {
                    return Ok(count);
                } else {
                    while !self.read_ready()? {}
                }
            }

            *byte = self.read_tx_rx_fifo().data();
        }

        Ok(buf.len())
    }
}

impl WriteReady for MmioUart<'_> {
    fn write_ready(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.read_channel_status().tx_fifo_full())
    }
}

impl Write for MmioUart<'_> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        // The implementation corresponds to Table 21-8 in the TRM, simplified
        // because we don't use interrupts.

        for (count, &byte) in buf.iter().enumerate() {
            // The contract for this function says that we must not block if
            // `WriteReady::write_ready` has returned true, so instead we have
            // to stop transmitting if the FIFO is full after the first byte.
            // In contrast, we need to block when we can't send the first byte
            // because the contract requires us to block until we can send at
            // least one byte.
            if !self.write_ready()? {
                if count > 0 {
                    return Ok(count);
                } else {
                    while !self.write_ready()? {}
                }
            }

            self.write_tx_rx_fifo(FifoRegister::builder().with_data(byte).build());
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        loop {
            let status = self.read_channel_status();
            if !status.tx_active() && status.tx_fifo_empty() {
                break;
            }
        }

        Ok(())
    }
}
