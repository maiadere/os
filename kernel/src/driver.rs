pub mod gpio;
pub mod uart0;

const ARE_PERIPHERALS_UPPER_HALF: bool = true;
const UPPER_HALF_OFFSET: u64 = 0xfffffffe00000000;
const REGISTER_BASE_OFFSET: u64 = if ARE_PERIPHERALS_UPPER_HALF {
    UPPER_HALF_OFFSET
} else {
    0
};
