//! # MMU Configuration

// We configure the MMU with a 4 GB virtual address space and 4 KB granules,
// using only TTBR0; this means that we have a Level 1 table of 4 entries
// (corresponding to 1 GB each), 512 Level 2 entries per table corresponding to
// 2 MB each, and 512 Level 3 entries per table corresponding to 4 KB each.
//
// The device's memory map is documented in Figure 10-1 of the TRM.
//
// The MMU configuration provides memory protection and fault detection:
//
// - Executable regions are read-only and writable regions are non-executable.
// - Guard pages are placed around stacks to detect overflow.
// - Unused virtual address space is left unmapped to catch invalid accesses.
//
// The implementation only supports 48-bit physical addresses.
//
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
use arbitrary_int::{u3, u18, u27, u36};
use bitbybit::{bitenum, bitfield};

const LEVEL1_TABLE_ENTRIES: usize = 4;
const LEVEL2_TABLE_ENTRIES: usize = 512;
const LEVEL3_TABLE_ENTRIES: usize = 512;

const LEVEL2_BLOCK_SIZE: u64 = 0x200000; // 2 MB
const LEVEL3_PAGE_SIZE: u64 = 0x1000; // 4 KB

/// A translation table with N entries.
///
/// Each entry can be a block descriptor, a table descriptor, or a page
/// descriptor.
#[repr(C, align(4096))]
struct Table<const N: usize> {
    table: [u64; N],
}

/// Table descriptor for use in translation tables.
///
/// See Section D8.3 in the Architecture Reference Manual.
#[bitfield(u64, default = 0)]
struct TableDescriptor {
    #[bits(63..=63, rw)]
    non_secure: bool,
    #[bits(61..=62, rw)]
    access_permissions: AccessPermissions,
    #[bits(60..=60, rw)]
    execute_never: bool,
    #[bits(59..=59, rw)]
    privileged_execute_never: bool,
    #[bits(12..=47, rw)]
    table_address: u36,
    #[bit(10, rw)]
    access_flag: bool,
    #[bit(1, rw)]
    table_descriptor: bool,
    #[bit(0, rw)]
    valid: bool,
}

impl TableDescriptor {
    const fn table(address: u64) -> u64 {
        Self::DEFAULT
            .with_table_address(TableDescriptor::to_table_address(address))
            .with_access_flag(true)
            .with_table_descriptor(true)
            .with_valid(true)
            .raw_value()
    }

    /// Translates a full address to a table address.
    const fn to_table_address(address: u64) -> u36 {
        u36::extract_u64(address, 12)
    }
}

/// Block or page descriptor for use in translation tables.
///
/// See Section D8.3 in the Architecture Reference Manual.
#[bitfield(u64, default = 0)]
struct LeafDescriptor {
    #[bit(54, rw)]
    unprivileged_execute_never: bool,
    #[bit(53, rw)]
    privileged_execute_never: bool,
    #[bit(52, rw)]
    contiguous: bool,
    #[bits(30..=47, rw)]
    level1_block_address: u18,
    #[bits(21..=47, rw)]
    level2_block_address: u27,
    #[bits(12..=47, rw)]
    level3_page_address: u36,
    #[bit(10, rw)]
    access_flag: bool,
    #[bits(8..=9, rw)]
    shareability: Shareability,
    #[bits(6..=7, rw)]
    access_permissions: AccessPermissions,
    #[bit(5, rw)]
    non_secure: bool,
    #[bits(2..=4, rw)]
    attribute_index: u3,
    #[bit(1, rw)]
    page_descriptor: bool,
    #[bit(0, rw)]
    valid: bool,
}

impl LeafDescriptor {
    const fn level1_data(address: u64) -> u64 {
        Self::data_block()
            .with_level1_block_address(Self::to_level1_block_address(address))
            .raw_value()
    }

    const fn level1_device(address: u64) -> u64 {
        Self::device_block()
            .with_level1_block_address(Self::to_level1_block_address(address))
            .raw_value()
    }

    const fn level2_data(address: u64) -> u64 {
        Self::data_block()
            .with_level2_block_address(Self::to_level2_block_address(address))
            .raw_value()
    }

    const fn level3_code(address: u64) -> u64 {
        Self::page()
            .with_level3_page_address(Self::to_level3_page_address(address))
            .with_access_flag(true)
            .with_shareability(Shareability::Outer)
            .with_access_permissions(AccessPermissions::PrivilegedRead)
            .with_non_secure(true)
            .with_attribute_index(u3::new(0)) // see MAIR_EL1 setup
            .raw_value()
    }

    const fn level3_data(address: u64, access_permissions: AccessPermissions) -> u64 {
        Self::page()
            .with_level3_page_address(Self::to_level3_page_address(address))
            .with_unprivileged_execute_never(true)
            .with_privileged_execute_never(true)
            .with_access_flag(true)
            .with_shareability(Shareability::Outer)
            .with_access_permissions(access_permissions)
            .with_non_secure(true)
            .with_attribute_index(u3::new(0)) // see MAIR_EL1 setup
            .raw_value()
    }

    const fn data_block() -> Self {
        Self::new_with_raw_value(0b01)
            .with_unprivileged_execute_never(true)
            .with_privileged_execute_never(true)
            .with_access_flag(true)
            .with_shareability(Shareability::Outer)
            .with_access_permissions(AccessPermissions::PrivilegedReadWrite)
            .with_non_secure(true)
            .with_attribute_index(u3::new(0)) // see MAIR_EL1 setup
    }

    const fn device_block() -> Self {
        Self::block()
            .with_unprivileged_execute_never(true)
            .with_privileged_execute_never(true)
            .with_access_flag(true)
            .with_shareability(Shareability::Outer)
            .with_access_permissions(AccessPermissions::UnprivilegedReadWrite)
            .with_non_secure(true)
            .with_attribute_index(u3::new(1)) // see MAIR_EL1 setup
    }

    const fn block() -> Self {
        Self::DEFAULT.with_valid(true)
    }

    const fn page() -> Self {
        Self::DEFAULT.with_page_descriptor(true).with_valid(true)
    }

    /// Translates a full address to a level 1 block address.
    const fn to_level1_block_address(address: u64) -> u18 {
        u18::extract_u64(address, 30)
    }

    /// Translates a full address to a level 2 block address.
    const fn to_level2_block_address(address: u64) -> u27 {
        u27::extract_u64(address, 21)
    }

    /// Translates a full address to a level 3 page address.
    const fn to_level3_page_address(address: u64) -> u36 {
        u36::extract_u64(address, 12)
    }
}

const INVALID_DESCRIPTOR: u64 = 0;

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

/// Our Level 1 translation table.
///
/// Each entry corresponds to 1 GB of memory.
static LEVEL1_TABLE: spin::Once<Table<LEVEL1_TABLE_ENTRIES>> = spin::Once::new();

fn level1_table() -> &'static Table<LEVEL1_TABLE_ENTRIES> {
    LEVEL1_TABLE.call_once(|| {
        Table {
            table: [
                // 1 GB (0x0000_0000 - 0x3FFF_FFFF): DDR
                TableDescriptor::table(level2_table() as *const _ as u64),
                // 1 GB (0x4000_0000 - 0x7FFF_FFFF): DDR
                LeafDescriptor::level1_data(0x4000_0000),
                // 1 GB (0x8000_0000 - 0xBFFF_FFFF): Devices
                LeafDescriptor::level1_device(0x8000_0000),
                // 1 GB (0xC000_0000 - 0xFFFF_FFFF): Devices. This isn't completely
                // correct because there are reserved regions in between, but it
                // shouldn't be critical at the moment either.
                LeafDescriptor::level1_device(0xc000_0000),
            ],
        }
    })
}

/// Our Level 2 translation table.
///
/// Each entry corresponds to 2 MB of memory.
static LEVEL2_TABLE: spin::Once<Table<LEVEL2_TABLE_ENTRIES>> = spin::Once::new();

fn level2_table() -> &'static Table<LEVEL2_TABLE_ENTRIES> {
    unsafe extern "C" {
        static __heap_start: u8;
        static __heap_end: u8;
    }

    LEVEL2_TABLE.call_once(|| {
        let heap_start_address = unsafe { &__heap_start as *const _ as u64 };
        Table {
            table: core::array::from_fn(|i| {
                let address = i as u64 * LEVEL2_BLOCK_SIZE;
                if address < heap_start_address {
                    TableDescriptor::table(level3_table(i) as *const _ as u64)
                } else {
                    LeafDescriptor::level2_data(address)
                }
            }),
        }
    })
}

/// Our Level 3 translation tables.
///
/// Each entry corresponds to 4 KB of memory.
static LEVEL3_TABLES: [spin::Once<Table<LEVEL3_TABLE_ENTRIES>>; LEVEL2_TABLE_ENTRIES] =
    [const { spin::Once::new() }; LEVEL2_TABLE_ENTRIES];

fn level3_table(i: usize) -> &'static Table<LEVEL3_TABLE_ENTRIES> {
    unsafe extern "C" {
        static __text_start: u8;
        static __text_end: u8;
        static __data_start: u8;
    }

    LEVEL3_TABLES[i].call_once(|| {
        let text_start_address = unsafe { &__text_start as *const _ as u64 };
        let text_end_address = unsafe { &__text_end as *const _ as u64 };
        let data_start_address = unsafe { &__data_start as *const _ as u64 };
        Table {
            table: core::array::from_fn(|j| {
                let address = i as u64 * LEVEL2_BLOCK_SIZE + j as u64 * LEVEL3_PAGE_SIZE;
                if address < text_start_address || guard_pages().contains(&address) {
                    // Prevent access to guard pages and unused memory
                    INVALID_DESCRIPTOR
                } else if address >= text_start_address && address < text_end_address {
                    LeafDescriptor::level3_code(address)
                } else {
                    LeafDescriptor::level3_data(
                        address,
                        if address < data_start_address {
                            AccessPermissions::PrivilegedRead
                        } else {
                            AccessPermissions::PrivilegedReadWrite
                        },
                    )
                }
            }),
        }
    })
}

static GUARD_PAGES: spin::Once<[u64; 5]> = spin::Once::new();

fn guard_pages() -> &'static [u64] {
    unsafe extern "C" {
        static __stack_start: u8;
        static __stack0_end: u8;
        static __stack1_end: u8;
        static __stack2_end: u8;
        static __stack3_end: u8;
    }

    GUARD_PAGES.call_once(|| unsafe {
        [
            &__stack_start as *const _ as u64,
            &__stack0_end as *const _ as u64,
            &__stack1_end as *const _ as u64,
            &__stack2_end as *const _ as u64,
            &__stack3_end as *const _ as u64,
        ]
    })
}

/// Configure the MMU and enable it.
///
/// Use a flat memory mapping, i.e., virtual addresses are equal to physical
/// addresses. This means that the MMU is only used to enforce memory
/// protection and for caching.
pub fn enable() {
    // Configure the location of our Level 1 table.
    TTBR0_EL1.set(level1_table() as *const _ as u64);

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
