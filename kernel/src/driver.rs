pub mod gpio;
pub mod timer;
pub mod uart0;
pub mod videocore;

const ARE_PERIPHERALS_UPPER_HALF: bool = true;
const UPPER_HALF_OFFSET: u64 = 0xffff_fffe_0000_0000;

const PERIPHERAL_BASE: u64 = 0xfe00_0000
    + if ARE_PERIPHERALS_UPPER_HALF {
        UPPER_HALF_OFFSET
    } else {
        0
    };
