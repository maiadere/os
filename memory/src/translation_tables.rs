use crate::descriptors::{PageDescriptor, TableDescriptor};

/// Big monolithic struct for storing the translation tables. Individual levels must be 64 KiB
/// aligned, so the lvl3 is put first.
#[repr(C)]
#[repr(align(65536))]
pub struct FixedSizeTranslationTable<const NUM_TABLES: usize> {
    /// Page descriptors, covering 64 KiB windows per entry.
    lvl3: [[PageDescriptor; 8192]; NUM_TABLES],

    /// Table descriptors, covering 512 MiB windows.
    lvl2: [TableDescriptor; NUM_TABLES],
}
pub const NUM_LVL2_TABLES: usize = 16;

/// A translation table type for the kernel space.
pub type KernelTranslationTable = FixedSizeTranslationTable<NUM_LVL2_TABLES>;

impl<const NUM_TABLES: usize> FixedSizeTranslationTable<NUM_TABLES> {
    /// Create an instance.
    pub const fn new() -> Self {
        // Can't have a zero-sized address space.
        assert!(NUM_TABLES > 0);

        Self {
            lvl3: [[PageDescriptor::new_zeroed(); 8192]; NUM_TABLES],
            lvl2: [TableDescriptor::new_zeroed(); NUM_TABLES],
        }
    }

    /// Iterates over all static translation table entries and fills them at once.
    ///
    /// # Safety
    ///
    /// - Modifies a `static mut`. Ensure it only happens from here.
#[allow(static_mut_refs)]
    pub unsafe fn populate_tt_entries(&self) -> Result<(), &'static str> {
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
        Ok(())
    }
}
