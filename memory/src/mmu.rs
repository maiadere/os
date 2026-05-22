use core::arch;

use crate::descriptors::{PageDescriptor, TableDescriptor};
use aarch64_cpu::registers::{self, ReadWriteable};
use aarch64_cpu::registers::{MAIR_EL1, TCR_EL1, TTBR0_EL1, TTBR1_EL1};
use registers::Writeable;

pub enum MemoryAttribute {
    Device,
    Memory,
}

impl MemoryAttribute {
    pub fn to_mair_idx(&self) -> [bool; 3] {
        match self {
            Self::Memory => [false, false, false],
            Self::Device => [false, false, true],
        }
    }
}

// addresses for accessing the mmu translation tables
// low virtual addresses
const EL0_L2_TRANSLATION_TABLE_BASE_ADDRESS: u64 = 0x3b1f_0000;
// high virtual addresses
const EL1_L2_TRANSLATION_TABLE_BASE_ADDRESS: u64 = 0x3b1f_0080;
// start address of the low virtual address L3 page tables (which are contiguous)
const EL0_L3_TRANSLATION_TABLE_BASE_ADDRESS: u64 = 0x3b20_0000;
// start address of the high virtual address L3 page tables (which are contiguous)
const EL1_L3_TRANSLATION_TABLE_BASE_ADDRESS: u64 = 0x3b30_0000;

const UPPER_31_BITS: u64 = 0xfffffffe00000000;
const LOWER_33_BITS: u64 = 0x1FFFFFFFF;

// size of a single translation granule in bytes
const PAGE_SIZE: u64 = 0x1_0000;

/// sets up registers controlling address translations
/// safety: this modifies registers controlling the mmu, all bets are off
unsafe fn set_up_translation_control_registers() {
    // memory type indirection:
    // idx 0 is normal memory
    // idx 1 is device memory
    MAIR_EL1.write(
        MAIR_EL1::Attr0_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr0_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr1_Device::nonGathering_nonReordering_noEarlyWriteAck,
    );
    //granule size and address lengths
    TCR_EL1.write(
        TCR_EL1::T0SZ.val(31)
            + TCR_EL1::T1SZ.val(31)
            + TCR_EL1::IPS::Bits_44
            + TCR_EL1::TG0::KiB_64
            + TCR_EL1::TG1::KiB_64,
    );
    // Point MMU to the tables
    TTBR0_EL1.set_baddr(EL0_L2_TRANSLATION_TABLE_BASE_ADDRESS);
    TTBR1_EL1.set_baddr(EL1_L2_TRANSLATION_TABLE_BASE_ADDRESS);
}

/// pulls out the translation table offsets from the virtual address
/// assumes 64 KiB translation granules
/// returns (l1, l2, l3) offsets
/// must have the upper bits that select between EL1/EL0 be either all 1 or all 0
pub fn virtual_addr_to_tt_offsets(virtual_address: u64) -> (u64, u64, u64) {
    let l3_offset = (virtual_address >> 16) & 0b1111111111111;
    let l2_offset = (virtual_address >> 29) & 0b1111111111111;
    let l1_offset = (virtual_address >> 42) & 0b11111;
    (l1_offset, l2_offset, l3_offset)
}

/// for selecting whether the write to the translation tables is done
/// via their physical address or their upper half mapping
#[derive(Debug, Eq, PartialEq)]
enum TT_access {
    Identity,
    UpperHalf,
}

/// updates the page tables of the mmu
/// to map a range of virtual addresses to a range of physical addresses
/// safety: this modifies the way addresses are translated, all bets are off
/// all addresses should be aligned to 64 KiB (i. e. 0x1_0000 bytes)
unsafe fn map_address_range(
    virtual_address_start: u64,
    physical_address_start: u64,
    num_pages: u64,
    memory_attribute: MemoryAttribute,
    tt_access: TT_access,
) {
    assert_eq!(virtual_address_start % PAGE_SIZE, 0);
    assert_eq!(physical_address_start % PAGE_SIZE, 0);

    let is_upper_half = (virtual_address_start & UPPER_31_BITS) == UPPER_31_BITS;

    for i in 0..num_pages {
        let virtual_addr = (virtual_address_start & LOWER_33_BITS) + i * 0x1_0000;
        let phys_addr = physical_address_start + i * PAGE_SIZE;

        let (_, l2_offset, l3_offset) = virtual_addr_to_tt_offsets(virtual_addr);

        let l3_base_addr = if is_upper_half {
            EL1_L3_TRANSLATION_TABLE_BASE_ADDRESS
        } else {
            EL0_L3_TRANSLATION_TABLE_BASE_ADDRESS
        };

        let l3_base_addr = match tt_access {
            TT_access::Identity => l3_base_addr,
            TT_access::UpperHalf => l3_base_addr + UPPER_31_BITS,
        };
        let page_table_entry_addr = l3_base_addr + l2_offset * PAGE_SIZE + l3_offset * 8;

        let page_descriptor = PageDescriptor {
            uxn: false,
            pxn: false,
            contiguous: false,
            dirty_bit: false,
            not_global: false,
            access_flag: true,
            shareability: [false; 2],
            access_permission: [false; 2],
            non_secure: false,
            attributes_index: memory_attribute.to_mair_idx(),
            address: (phys_addr >> 16) as u32,
            upper_address: [false; 4],
            valid: true,
        };

        unsafe {
            (page_table_entry_addr as *mut u64).write_volatile(page_descriptor.bits());
        }
    }
}

unsafe fn mark_address_range_invalid(
    virtual_address_start: u64,
    num_pages: u64,
    tt_access: TT_access,
) {
    assert_eq!(virtual_address_start % PAGE_SIZE, 0);

    let is_upper_half = (virtual_address_start & UPPER_31_BITS) == UPPER_31_BITS;

    for i in 0..num_pages {
        let virtual_addr = (virtual_address_start & LOWER_33_BITS) + i * 0x1_0000;

        let (_, l2_offset, l3_offset) = virtual_addr_to_tt_offsets(virtual_addr);

        let l3_base_addr = if is_upper_half {
            EL1_L3_TRANSLATION_TABLE_BASE_ADDRESS
        } else {
            EL0_L3_TRANSLATION_TABLE_BASE_ADDRESS
        };

        let l3_base_addr = match tt_access {
            TT_access::Identity => l3_base_addr,
            TT_access::UpperHalf => l3_base_addr + UPPER_31_BITS,
        };
        let page_table_entry_addr = l3_base_addr + l2_offset * PAGE_SIZE + l3_offset * 8;

        let page_descriptor = PageDescriptor {
            uxn: false,
            pxn: false,
            contiguous: false,
            dirty_bit: false,
            not_global: false,
            access_flag: true,
            shareability: [false; 2],
            access_permission: [false; 2],
            non_secure: false,
            attributes_index: [false; 3],
            address: 0,
            upper_address: [false; 4],
            valid: false,
        };

        unsafe {
            (page_table_entry_addr as *mut u64).write_volatile(page_descriptor.bits());
        }
    }
}

pub unsafe fn enable_mmu() {
    unsafe {
        set_up_translation_control_registers();
    }

    unsafe {
        create_initial_mappings();
    };

    // switch mmu on
    aarch64_cpu::asm::barrier::isb(aarch64_cpu::asm::barrier::SY);
    registers::SCTLR_EL1.modify(registers::SCTLR_EL1::M::SET);
    aarch64_cpu::asm::barrier::isb(aarch64_cpu::asm::barrier::SY);
}

unsafe fn populate_l2_tables() {
    for i in 0..16 {
        let table_descriptor = TableDescriptor {
            ns_table: false,
            ap_table: [false; 2],
            uxn_table: false,
            pxn_table: false,
            upper_address: [false; 4],
            valid: true,
            address: (EL0_L3_TRANSLATION_TABLE_BASE_ADDRESS as u32 + i * PAGE_SIZE as u32) >> 16,
        };

        unsafe {
            ((EL0_L2_TRANSLATION_TABLE_BASE_ADDRESS + i as u64 * 8) as *mut u64)
                .write_volatile(table_descriptor.bits());
        }
    }

    for i in 0..16 {
        let table_descriptor = TableDescriptor {
            ns_table: false,
            ap_table: [false; 2],
            uxn_table: false,
            pxn_table: false,
            upper_address: [false; 4],
            valid: true,
            address: (EL1_L3_TRANSLATION_TABLE_BASE_ADDRESS as u32 + i * PAGE_SIZE as u32) >> 16,
        };

        unsafe {
            ((EL1_L2_TRANSLATION_TABLE_BASE_ADDRESS + i as u64 * 8) as *mut u64)
                .write_volatile(table_descriptor.bits());
        }
    }
}

// we have a total of 15168 pages available to use in the first gibibyte of memory
// WARNING: not all of those pages can be used,
// notably the last 33 pages hold the translation tables themselves
const TOTAL_PAGE_COUNT: u64 = 15168;
// NOTE: 1 MiB is 16 pages

// how many pages of memory the kernel gets for code and its stack
const KERNEL_PAGE_COUNT: u64 = 16;
// how much memory that corresponds to
const KERNEL_MEMORY: u64 = KERNEL_PAGE_COUNT * PAGE_SIZE;

// how many pages of memory the userspace gets for code + stack + heap
const USER_PAGE_COUNT: u64 = 64;
pub const USER_MEMORY: u64 = USER_PAGE_COUNT * PAGE_SIZE;

pub unsafe fn create_initial_mappings() {
    unsafe {
        populate_l2_tables();
    }
    // initalize tables to all invalid entries
    unsafe {
        mark_address_range_invalid(0, TOTAL_PAGE_COUNT, TT_access::Identity);
        mark_address_range_invalid(0 + UPPER_31_BITS, TOTAL_PAGE_COUNT, TT_access::Identity);
    }
    // identity mapping the kernel code and stack to maintain execution after starting the mmu
    unsafe {
        map_address_range(
            0x0000,
            0x0000,
            KERNEL_PAGE_COUNT,
            MemoryAttribute::Memory,
            TT_access::Identity,
        );
    }

    // upper half videocore sdram mapping (0x3b40_0000 - 0x4000_0000)
    unsafe {
        map_address_range(
            0x3b40_0000 + UPPER_31_BITS,
            0x3b40_0000,
            0x4C0,
            MemoryAttribute::Device,
            TT_access::Identity,
        );
    }

    // upper half for kernel code + stack
    unsafe {
        map_address_range(
            0x0000 + UPPER_31_BITS,
            0x0000,
            KERNEL_PAGE_COUNT,
            MemoryAttribute::Memory,
            TT_access::Identity,
        );
    }

    // upper half peripheral mapping
    unsafe {
        map_address_range(
            0xfc00_0000 + UPPER_31_BITS,
            0xfc00_0000,
            0x400,
            MemoryAttribute::Device,
            TT_access::Identity,
        );
    }

    // upper half translation table mapping
    unsafe {
        map_address_range(
            EL1_L3_TRANSLATION_TABLE_BASE_ADDRESS + UPPER_31_BITS,
            EL1_L3_TRANSLATION_TABLE_BASE_ADDRESS,
            16,
            MemoryAttribute::Memory,
            TT_access::Identity,
        );
    }
    unsafe {
        map_address_range(
            EL0_L3_TRANSLATION_TABLE_BASE_ADDRESS + UPPER_31_BITS,
            EL0_L3_TRANSLATION_TABLE_BASE_ADDRESS,
            16,
            MemoryAttribute::Memory,
            TT_access::Identity,
        );
    }
}
// map user space memory into virtual addresses starting at 0
// and the physical memory put after what is mapped for the kernel
pub unsafe fn post_boot_mappings() {
    // changing the entries requires doing it in a break-before-make sequence
    // refer to armv8 docs section D4.9.1

    //break old mapping
    unsafe {
        mark_address_range_invalid(0x0000, KERNEL_PAGE_COUNT, TT_access::UpperHalf);
    }
    //ensure visibility of changes
    aarch64_cpu::asm::barrier::dsb(aarch64_cpu::asm::barrier::SY);
    unsafe {
        arch::asm!("tlbi vmalle1");
    }
    aarch64_cpu::asm::barrier::dsb(aarch64_cpu::asm::barrier::SY);

    // make new mapping
    unsafe {
        map_address_range(
            0x0000,
            KERNEL_MEMORY,
            USER_PAGE_COUNT,
            MemoryAttribute::Memory,
            TT_access::UpperHalf,
        )
    }
    //ensure visibility
    aarch64_cpu::asm::barrier::dsb(aarch64_cpu::asm::barrier::SY);
}
