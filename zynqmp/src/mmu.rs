//! # MMU Configuration

// The implementation is based on the Zynq UltraScale+ Device Technical
// Reference Manual (UG1085) [1], the AArch64 memory management Guide [2], the
// AArch64 memory management examples [3], and the Arm® Architecture Reference
// Manual, for A-profile architecture [4].
//
// [1] https://docs.amd.com/v/u/en-US/ug1085-zynq-ultrascale-trm
// [2] https://developer.arm.com/documentation/101811/0104
// [3] https://developer.arm.com/documentation/102416/0100
// [4] https://developer.arm.com/documentation/ddi0487/latest/

use core::arch::asm;

use aarch64_cpu::{
    asm::barrier::{SY, dsb, isb},
    registers::*,
};
use arbitrary_int::{u3, u27};
use bitbybit::{bitenum, bitfield};

const LEVEL1_TABLE_ENTRIES: usize = 4;

/// A translation table with N entries.
///
/// Each entry can be a block descriptor, a table descriptor, or a page
/// descriptor.
#[repr(C, align(4096))]
struct Table<const N: usize> {
    table: [u64; N],
}

/// Block descriptor for use in translation tables.
///
/// See Section D8.3 in the Architecture Reference Manual.
#[bitfield(u64)]
struct BlockDescriptor {
    #[bit(54, rw)]
    unprivileged_execute_never: bool,
    #[bit(53, rw)]
    privileged_execute_never: bool,
    #[bit(52, rw)]
    contiguous: bool,
    #[bits(21..=47, rw)]
    shifted_address: u27,
    #[bit(10, rw)]
    access_flag: bool,
    #[bits(8..=9, rw)]
    shareability: Shareability,
    #[bits(6..=7, rw)]
    access_permissions: AccessPermissions,
    #[bit(5, rw)]
    not_secure: bool,
    #[bits(2..=4, rw)]
    attribute_index: u3,
    #[bit(1, rw)]
    table_descriptor: bool,
    #[bit(0, rw)]
    valid: bool,
}

#[bitenum(u2, exhaustive = true)]
#[allow(dead_code)]
enum Shareability {
    None = 0b00,
    Reserved = 0b01,
    Outer = 0b10,
    Inner = 0b11,
}

#[bitenum(u2, exhaustive = true)]
#[allow(dead_code)]
enum AccessPermissions {
    PrivilegedReadWrite = 0b00,
    UnprivilegedReadWrite = 0b01,
    PrivilegedRead = 0b10,
    UnprivilegedRead = 0b11,
}

/// Translates a full address to the shifted address expected in a
/// `BlockDescriptor`.
const fn shift_address(address: u64) -> u27 {
    u27::extract_u64(address, 21)
}

/// Our Level 1 translation table.
///
/// Each entry corresponds to 1GB of memory.
const LEVEL1_TABLE: Table<LEVEL1_TABLE_ENTRIES> = Table {
    table: [
        // 1GB (0x0000_0000 - 0x3FFF_FFFF): DDR
        BlockDescriptor::new_with_raw_value(0)
            .with_shifted_address(shift_address(0x0000_0000))
            .with_access_flag(true)
            .with_shareability(Shareability::Outer)
            .with_access_permissions(AccessPermissions::PrivilegedReadWrite)
            .with_not_secure(true)
            .with_attribute_index(u3::new(0)) // see MAIR_EL1 setup
            .with_valid(true)
            .raw_value(),
        // 1GB (0x4000_0000 - 0x7FFF_FFFF): DDR
        BlockDescriptor::new_with_raw_value(0)
            .with_shifted_address(shift_address(0x4000_0000))
            .with_access_flag(true)
            .with_shareability(Shareability::Outer)
            .with_access_permissions(AccessPermissions::PrivilegedReadWrite)
            .with_not_secure(true)
            .with_attribute_index(u3::new(0)) // see MAIR_EL1 setup
            .with_valid(true)
            .raw_value(),
        // 1GB (0x8000_0000 - 0xBFFF_FFFF): Devices
        BlockDescriptor::new_with_raw_value(0)
            .with_shifted_address(shift_address(0x8000_0000))
            .with_unprivileged_execute_never(true)
            .with_privileged_execute_never(true)
            .with_access_flag(true)
            .with_shareability(Shareability::Outer)
            .with_access_permissions(AccessPermissions::UnprivilegedReadWrite)
            .with_not_secure(true)
            .with_attribute_index(u3::new(1)) // see MAIR_EL1 setup
            .with_valid(true)
            .raw_value(),
        // 1GB (0xC000_0000 - 0xFFFF_FFFF): Devices. This isn't completely
        // correct because there are reserved regions in between, but it
        // shouldn't be critical at the moment either.
        BlockDescriptor::new_with_raw_value(0)
            .with_shifted_address(shift_address(0xc000_0000))
            .with_unprivileged_execute_never(true)
            .with_privileged_execute_never(true)
            .with_access_flag(true)
            .with_shareability(Shareability::Outer)
            .with_access_permissions(AccessPermissions::UnprivilegedReadWrite)
            .with_not_secure(true)
            .with_attribute_index(u3::new(1)) // see MAIR_EL1 setup
            .with_valid(true)
            .raw_value(),
    ],
};

/// Configure the MMU and enable it.
///
/// Use a flat memory mapping, i.e., virtual addresses are equal to physical
/// addresses. This means that the MMU is only used to enforce memory
/// protection and for caching.
pub fn enable() {
    // We configure the MMU with 32-bit virtual addresses and 4KB granules,
    // using only TTBR0; this means that we have a Level 1 table of 4 entries
    // (corresponding to 1GB each), 512 Level 2 entries per table corresponding
    // to 2MB each, and 512 Level 3 entries per table corresponding to 4KB
    // each.
    //
    // The device's memory map is documented in Figure 10-1 of the TRM.

    // Configure the location of our Level 1 table.
    TTBR0_EL1.set(&LEVEL1_TABLE as *const _ as u64);

    // Configure the available sets of memory attributes: set 0 is for normal
    // memory, set 1 for device memory.
    MAIR_EL1.write(
        MAIR_EL1::Attr0_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr0_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr1_Device::nonGathering_nonReordering_noEarlyWriteAck,
    );

    // Configure the translation regime as described above.
    TCR_EL1.write(
        TCR_EL1::T0SZ.val(32)
            + TCR_EL1::EPD0::EnableTTBR0Walks
            + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::SH0::Inner
            + TCR_EL1::TG0::KiB_4
            + TCR_EL1::EPD1::DisableTTBR1Walks
            + TCR_EL1::IPS::Bits_32,
    );

    // Invalidate the TLB and instruction caches.
    unsafe {
        asm!("tlbi vmalle1", "ic iallu");
    }

    // Data and instruction synchronization barrier before we enable the MMU.
    dsb(SY);
    isb(SY);

    // Now enable it.
    SCTLR_EL1.write(
        SCTLR_EL1::M::Enable
            + SCTLR_EL1::C::Cacheable
            + SCTLR_EL1::I::Cacheable
            + SCTLR_EL1::NTWI::DontTrap
            + SCTLR_EL1::NTWE::DontTrap,
    );
}
