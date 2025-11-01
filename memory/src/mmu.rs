use aarch64_cpu::registers;
use aarch64_cpu::registers::{MAIR_EL1, TCR_EL1, TTBR0_EL1, TTBR1_EL1};
use registers::Writeable;

use crate::translation_tables::{KernelTranslationTable};
pub enum MMUEnableError {
    AlreadyEnabled,
    Other(&'static str),
}

static mut TRANSLATION_TABLES: KernelTranslationTable = KernelTranslationTable::new();
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
        unsafe { TRANSLATION_TABLES.populate_tt_entries() };
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
        // L2 table descriptors
    fn configure_translation_control(&self) {
        // TCR_EL1 flags
        TCR_EL1.write(TCR_EL1::T0SZ.val(31));
        TCR_EL1.write(TCR_EL1::T1SZ.val(31));

        TCR_EL1.write(TCR_EL1::IPS::Bits_44);
        TCR_EL1.write(TCR_EL1::TG0::KiB_64);
        TCR_EL1.write(TCR_EL1::TG1::KiB_64);
    }
}
