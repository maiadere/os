fn bits_to_u64<const T: usize>(bits: &[bool; T]) -> u64 {
    bits.iter().fold(0, |acc, elem| acc * 2 + (*elem as u64))
}

/// Struct representing a table descriptor entry in the L0, L1 and L2 tables
/// assuming a 64 KiB granule size
struct TableDescriptor {
    ns_table: bool,
    ap_table: [bool; 2],
    uxn_table: bool,
    pxn_table: bool,
    address: u32,
    upper_address: [bool; 4],
    valid: bool,
}

impl TableDescriptor {
    /// returns a binary representation of the descriptor
    /// which can be inserted into a translation table
    fn bits(&self) -> u64 {
        ((self.ns_table as u64) << 63)
            | bits_to_u64(&self.ap_table) << 61
            | (self.uxn_table as u64) << 60
            | (self.pxn_table as u64) << 59
            | (self.address as u64) << 16
            | bits_to_u64(&self.upper_address) << 12
            | 0b10 // denotes a Table descriptor instead of a Block descriptor
            | self.valid as u64
    }
}

/// Struct representing a page descriptor entry in L3 tables
/// assuming a 64 KiB granule size
struct PageDescriptor {
    uxn: bool,
    pxn: bool,
    contiguous: bool,
    dirty_bit: bool,
    not_global: bool,
    access_flag: bool,
    shareability: [bool; 2],
    access_permission: [bool; 2],
    non_secure: bool,
    attributes_index: [bool; 3],
    address: u32,
    upper_address: [bool; 4],
    valid: bool,
}

impl PageDescriptor {
    /// returns a binary representation of the descriptor
    /// which can be inserted into a translation table
    fn bits(&self) -> u64 {
        ((self.uxn as u64) << 54)
            | ((self.pxn as u64) << 53)
            | ((self.contiguous as u64) << 52)
            | ((self.dirty_bit as u64) << 51)
            | ((self.address as u64) << 16)
            | bits_to_u64(&self.upper_address) << 12
            | ((self.not_global as u64) << 11)
            | ((self.access_flag as u64) << 10)
            | bits_to_u64(&self.shareability) << 8
            | bits_to_u64(&self.access_permission) << 6
            | ((self.non_secure as u64) << 5)
            | bits_to_u64(&self.attributes_index) << 2
            | 0b10 // must always be set to 1 for page descriptors in L3
            | (self.valid as u64)
    }
}
