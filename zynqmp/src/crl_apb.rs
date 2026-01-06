//! # CRL_APB Module: FPD Clock and Reset Control

// The implementation is baed on the Zynq UltraScale+ Devices Register Reference (UG1087) [1].
//
// [1] https://docs.amd.com/r/en-US/ug1087-zynq-ultrascale-registers/CRL_APB-Module

use bitbybit::bitfield;

/// Representation of the CRL_APB module registers.
///
/// FPD Clock and Reset control
#[derive(derive_mmio::Mmio)]
#[repr(C)]
pub struct CrlApb {
    /// SLVERR Error Signal Enable
    err_ctrl: u32,
    /// Interrupt Status and Clear
    ir_status: u32,
    /// Interrupt Mask
    #[mmio(PureRead)]
    ir_mask: u32,
    /// Interrupt Enable
    #[mmio(Write)]
    ir_enable: u32,
    /// Interrupt Disable
    #[mmio(Write)]
    ir_disable: u32,
    /// Reserved
    _reserved01: [u32; 2],
    /// CRL SLCR Write Register Protection Control
    crl_wprot: CrlWprot,
    /// IOPLL Clock Unit Control
    iopll_ctrl: u32,
    /// IOPLL Integer Helper Data Config
    iopll_cfg: u32,
    /// Fractional control for the PLL
    iopll_frac_cfg: u32,
    /// Reserved
    _reserved02: u32,
    /// RPLL Clock Unit Control
    rpll_ctrl: u32,
    /// RPLL Integer Helper Data Configuration
    rpll_cfg: u32,
    /// Fractional control for the PLL
    rpll_frac_cfg: u32,
    /// Reserved
    _reserved03: u32,
    /// LPD PLL Clocking Status
    pll_status: u32,
    /// IOPLL clock divider for distribution in FPD
    iopll_to_fpd_ctrl: u32,
    /// RPLL clock divider for distribution in FPD
    rpll_to_fpd_ctrl: u32,
    /// USB 3.0 Unit Clock Generator Control
    usb3_dual_ref_ctrl: u32,
    /// GEM 0 Clock Generator Control
    gem0_ref_ctrl: u32,
    /// GEM 1 Clock Generator Control
    gem1_ref_ctrl: u32,
    /// GEM 2 Clock Generator Config
    gem2_ref_ctrl: u32,
    /// GEM 3 Clock Generator Config
    gem3_ref_ctrl: u32,
    /// USB 0 Clock Generator Config
    usb0_bus_ref_ctrl: u32,
    /// USB 1 Clock Generator Config
    usb1_bus_ref_ctrl: u32,
    /// Quad-SPI Clock Generator Config
    qspi_ref_ctrl: u32,
    /// SDIO 0 Clock Generator Config
    sdio0_ref_ctrl: u32,
    /// SDIO 1 Clock Generator Config
    sdio1_ref_ctrl: u32,
    /// UART 0 Clock Generator Config
    uart0_ref_ctrl: u32,
    /// UART 1 Clock Generator Config
    uart1_ref_ctrl: u32,
    /// SPI 0 Clock Generator Config
    spi0_ref_ctrl: u32,
    /// SPI 1 Clock Generator Config
    spi1_ref_ctrl: u32,
    /// CAN 0 Clock Generator Config
    can0_ref_ctrl: u32,
    /// CAN 1 Clock Generator Config
    can1_ref_ctrl: u32,
    /// Reserved
    _reserved05: u32,
    /// RPU MPCore and OCM Clock Generator Config
    cpu_r5_ctrl: u32,
    /// Reserved
    _reserved06: [u32; 2],
    /// AXI Interface Clock Generator Config for LPD In/Outbound Switches
    iou_switch_ctrl: u32,
    /// CSU Clock Generator Config
    csu_pll_ctrl: u32,
    /// PCAP Clock Generator Config
    pcap_ctrl: u32,
    /// AXI Interface Clock Generator Config for LPD Main Switch
    lpd_switch_ctrl: u32,
    /// APB Interface Clock Generator Config for LPD IOP In/Outbound Switches
    lpd_lsbus_ctrl: u32,
    /// Debug Clock Generator Config in LPD
    dbg_lpd_ctrl: u32,
    /// NAND Clock Generator Config
    nand_ref_ctrl: u32,
    /// LPD DMA Clock Generator Config
    lpd_dma_ref_ctrl: u32,
    /// Reserved
    _reserved07: u32,
    /// PL 0 Clock Generator Config
    pl0_ref_ctrl: u32,
    /// PL 1 Clock Generator Config
    pl1_ref_ctrl: u32,
    /// PL 2 Clock Generator Config
    pl2_ref_ctrl: u32,
    /// PL 3 Clock Generator Config
    pl3_ref_ctrl: u32,
    /// PL Clock 0 Threshold Control and status
    pl0_thr_ctrl: u32,
    /// PL Clock 0 Count Value
    pl0_thr_cnt: u32,
    /// PL Clock 1 Threshold Control and status
    pl1_thr_ctrl: u32,
    /// PL Clock 1 Threshold Count Value
    pl1_thr_cnt: u32,
    /// PL Clock 2 Threshold Control and status
    pl2_thr_ctrl: u32,
    /// PL Clock 2 Threshold Count Value
    pl2_thr_cnt: u32,
    /// PL Clock 3 Threshold Control and status
    pl3_thr_ctrl: u32,
    /// Reserved
    _reserved08: [u32; 4],
    /// PL Clock 3 Threshold Count Value
    pl3_thr_cnt: u32,
    /// GEM TimeStamp Clock Generator Control
    gem_tsu_ref_ctrl: u32,
    /// Clock Generator Control
    dll_ref_ctrl: u32,
    /// PS SYSMON Clock Generator Control
    pssysmon_ref_ctrl: u32,
    /// Reserved
    _reserved09: [u32; 5],
    /// I2C 0 Clock Generator Control
    i2c0_ref_ctrl: u32,
    /// I2C 1 Clock Generator Control
    i2c1_ref_ctrl: u32,
    /// Timestamp Clock Generator Control
    timestamp_ref_ctrl: u32,
    /// Reserved
    _reserved10: u32,
    /// Safety Endpoint Connectivity Check
    safety_chk: u32,
    /// Reserved
    _reserved11: [u32; 3],
    /// Clock Monitor Interrupt Status
    clkmon_status: u32,
    /// Clock Monitor Interrupt Mask
    #[mmio(PureRead)]
    clkmon_mask: u32,
    /// Clock Monitor Interrupt Enable
    #[mmio(Write)]
    clkmon_enable: u32,
    /// Clock Monitor Interrupt Disable
    #[mmio(Write)]
    clkmon_disable: u32,
    /// Clock Monitor Interrupt Trigger
    #[mmio(Write)]
    clkmon_trigger: u32,
    /// Reserved
    _reserved12: [u32; 3],
    /// Upper Clock Comparison Threshold
    chkr0_clka_upper: u32,
    /// Lower Clock Comparison Threshold
    chkr0_clka_lower: u32,
    /// CLK B Counting Value
    chkr0_clkb_cnt: u32,
    /// Clock Checker 0 Control
    chkr0_ctrl: u32,
    /// Upper Clock Comparison Threshold
    chkr1_clka_upper: u32,
    /// Lower Clock Comparison Threshold
    chkr1_clka_lower: u32,
    /// CLK B Counting Value
    chkr1_clkb_cnt: u32,
    /// Clock Checker 1 Control
    chkr1_ctrl: u32,
    /// Upper Clock Comparison Threshold
    chkr2_clka_upper: u32,
    /// Lower Clock Comparison Threshold
    chkr2_clka_lower: u32,
    /// CLK B Counting Value
    chkr2_clkb_cnt: u32,
    /// Clock Checker 2 Control
    chkr2_ctrl: u32,
    /// Upper Clock Comparison Threshold
    chkr3_clka_upper: u32,
    /// Lower Clock Comparison Threshold
    chkr3_clka_lower: u32,
    /// CLK B Counting Value
    chkr3_clkb_cnt: u32,
    /// Clock Checker 3 Control
    chkr3_ctrl: u32,
    /// Upper Clock Comparison Threshold
    chkr4_clka_upper: u32,
    /// Lower Clock Comparison Threshold
    chkr4_clka_lower: u32,
    /// CLK B Counting Value
    chkr4_clkb_cnt: u32,
    /// Clock Checker 4 Control
    chkr4_ctrl: u32,
    /// Upper Clock Comparison Threshold
    chkr5_clka_upper: u32,
    /// Lower Clock Comparison Threshold
    chkr5_clka_lower: u32,
    /// CLK B Counting Value
    chkr5_clkb_cnt: u32,
    /// Clock Checker 5 Control
    chkr5_ctrl: u32,
    /// Upper Clock Comparison Threshold
    chkr6_clka_upper: u32,
    /// Lower Clock Comparison Threshold
    chkr6_clka_lower: u32,
    /// CLK B Counting Value
    chkr6_clkb_cnt: u32,
    /// Clock Checker 6 H723Control
    chkr6_ctrl: u32,
    /// Upper Clock Comparison Threshold
    chkr7_clka_upper: u32,
    /// Lower Clock Comparison Threshold
    chkr7_clka_lower: u32,
    /// CLK B Counting Value
    chkr7_clkb_cnt: u32,
    /// Clock Checker 7 Control
    chkr7_ctrl: u32,
    /// Reserved
    _reserved13: [u32; 9],
    /// Software controlled BOOT MODE
    boot_mode_user: u32,
    /// Hardware controlled BOOT MODE register
    boot_mode_por: u32,
    /// Reserved
    _reserved14: [u32; 3],
    /// PS_SRST_B Pin Control and Trigger
    reset_ctrl: ResetCtrl,
    /// Records the Reason for the Block-only Reset
    blockonly_rst: u32,
    /// Records the Reason for the Reset
    reset_reason: u32,
    /// Reserved
    _reserved15: [u32; 3],
    /// Software Reset of Ethernet GEM Controllers
    rst_lpd_iou0: u32,
    /// Reserved
    _reserved16: u32,
    /// IOP Software Reset Controls
    rst_lpd_iou2: u32,
    /// Software Reset Control for LPD System Elements
    rst_lpd_top: u32,
    /// Debug control for both the LPD and FPD
    rst_lpd_dbg: u32,
    /// Reserved
    _reserved17: [u32; 3],
    /// Used to control the mode pins after boot
    boot_pin_ctrl: u32,
    /// Reserved
    _reserved18: [u32; 7],
    /// Drive strength control 0 for DIO bank 3
    bank3_ctrl0: u32,
    /// Drive strength control 1 for DIO bank 3
    bank3_ctrl1: u32,
    /// Schmitt/CMOS input select for DIO bank 3
    bank3_ctrl2: u32,
    /// Pull-up/down select for DIO bank 3
    bank3_ctrl3: u32,
    /// Pull-up/down enable for DIO bank 3
    bank3_ctrl4: u32,
    /// Slew rate control for DIO bank 3
    bank3_ctrl5: u32,
    /// Voltage mode status for DIO bank 3
    #[mmio(PureRead)]
    bank3_status: u32,
}

#[bitfield(u32)]
pub struct CrlWprot {
    /// 0: Writes enabled
    /// 1: Writes disabled
    #[bit(0, rw)]
    active: bool,
}

#[bitfield(u32)]
pub struct ResetCtrl {
    /// Software reset
    #[bit(4, rw)]
    soft_reset: bool,
    /// 0: PS_SRST_B reset pin enabled
    /// 1: PS_SRST_B reset pin disabled
    #[bit(0, rw)]
    srst_dis: bool,
}

/// Creates a handle to the CRL_APB module.
pub const fn crl_apb() -> MmioCrlApb<'static> {
    unsafe { CrlApb::new_mmio_at(0xff5e_0000) }
}
