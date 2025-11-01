use aarch64_cpu::registers;
use aarch64_cpu::registers::{MAIR_EL1, TCR_EL1, TTBR0_EL1, TTBR1_EL1};
use registers::Writeable;

use crate::descriptors::{PageDescriptor, TableDescriptor};
pub enum MMUEnableError {
    AlreadyEnabled,
    Other(&'static str),
}
pub struct MemoryManagementUnit;

impl MemoryManagementUnit {
    fn set_up_mair(&self) {
        MAIR_EL1.write(MAIR_EL1::Attr0_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc);
        MAIR_EL1.write(MAIR_EL1::Attr0_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc);

        MAIR_EL1.write(MAIR_EL1::Attr1_Device::nonGathering_nonReordering_noEarlyWriteAck);
    }

    pub unsafe fn enable_mmu(&self) -> Result<(), MMUEnableError> {
        self.set_up_mair();
        // populate translation  tables
        self.populate_translation_tables();
        // Point MMU to the tables
        TTBR0_EL1.set_baddr(0x3B1F_0000);
        TTBR1_EL1.set_baddr(0x3B1F_0040);

        self.configure_translation_control();

        // switch mmu on
        aarch64_cpu::asm::barrier::isb(aarch64_cpu::asm::barrier::SY);
        registers::Writeable::write(&self::registers::SCTLR_EL1, registers::SCTLR_EL1::M::SET);
        aarch64_cpu::asm::barrier::isb(aarch64_cpu::asm::barrier::SY);
        Ok(())
    }
    fn populate_translation_tables(&self) {
        // L2 table descriptors
        for i in 0..16 {
            let table_descriptor = TableDescriptor {
                ns_table: false,
                ap_table: [false; 2],
                uxn_table: false,
                pxn_table: false,
                upper_address: [false; 4],
                valid: true,
                address: (0x3B20_0000 + i * 0x1_0000) >> 16,
            };

            unsafe {
                ((0x3b1f_0000 + i * 8) as *mut u64).write_volatile(table_descriptor.bits());
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
                address: (0x3B30_0000 + i * 0x1_0000) >> 16,
            };

            unsafe {
                ((0x3b1f_0040 + i * 8) as *mut u64).write_volatile(table_descriptor.bits());
            }
        }

        // L3 page descriptors
        // identity mapping 0x0 through 0xA_0000
        for i in 0..10 {
            let page_descriptor = PageDescriptor {
                uxn: false,
                pxn: false,
                contiguous: false,
                dirty_bit: false,
                not_global: false,
                access_flag: true,
                shareability: [false; 2],
                access_permission: [false; 2],
                non_secure: true, // assuming we run in insecure mode by default
                attributes_index: [false; 3],
                address: (i * 0x1_0000) >> 16,
                upper_address: [false; 4],
                valid: true,
            };

            unsafe {
                ((0x3B20_0000 + i * 8) as *mut u64).write_volatile(page_descriptor.bits());
            }
        }

        for i in 0..1024 {
            let page_descriptor = PageDescriptor {
                uxn: true,
                pxn: true,
                contiguous: false,
                dirty_bit: false,
                not_global: false,
                access_flag: true,
                shareability: [false; 2],
                access_permission: [false; 2],
                non_secure: true,
                attributes_index: [false, false, true],
                address: (0xfc00_0000 + i * 0x1_0000) >> 16,
                upper_address: [false; 4],
                valid: true,
            };
            unsafe {
                ((0x3b27_0000 + (0x1c00 + i) * 8) as *mut u64)
                    .write_volatile(page_descriptor.bits());
            }
        }
    }
    fn configure_translation_control(&self) {
        // TCR_EL1 flags
        TCR_EL1.write(TCR_EL1::T0SZ.val(31));
        TCR_EL1.write(TCR_EL1::T1SZ.val(31));

        TCR_EL1.write(TCR_EL1::IPS::Bits_44);
        TCR_EL1.write(TCR_EL1::TG0::KiB_64);
        TCR_EL1.write(TCR_EL1::TG1::KiB_64);
    }
}
