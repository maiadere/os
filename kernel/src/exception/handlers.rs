use core::ptr;
use core::ptr::slice_from_raw_parts;

use aarch64_cpu::registers::ESR_EL1;
use aarch64_cpu::registers::Readable;
use heapless::CString;
use heapless::String;
use heapless::{Vec, format};

use crate::driver::uart0;
use crate::log;
use crate::userspace;
use crate::userspace::check_userspace_pointer;
#[derive(Copy, Clone, Debug)]
#[repr(C)]

pub struct SavedRegisters {
    x0: u64,
    x1: u64,
    x2: u64,
    x3: u64,
    x4: u64,
    x5: u64,
    x6: u64,
    x7: u64,
    x8: u64,
    x9: u64,
    x10: u64,
    x11: u64,
    x12: u64,
    x13: u64,
    x14: u64,
    x15: u64,
    x16: u64,
    x17: u64,
    x18: u64,
    fp: u64,
    lr: u64,
    xzr: u64,
    esr: u64,
    far: u64,
}

#[unsafe(no_mangle)]
unsafe extern "C" fn _synchronous_kernel_exception(register_contents: *mut SavedRegisters) {
    // assumes uart is initialized
    uart0::write_str("kernel panic: \n");
    // safe as this should be set from the low level handler code
    let register_contents = unsafe { *register_contents };
    let Ok(formatted_contents) = format!(512; "{:?}\n", register_contents) else {
        panic!("could not format register contents, halting now\n");
    };
    uart0::write_str(&formatted_contents.as_str());
    panic!("halting now\n")
}

// syscall abi:
// the syscall number is the immediate of the svc instruction (svc #N), read from ESR_EL1.ISS
// arguments are passed in x0-x5, a return value is written into the saved x0,
// which the handler restores to the program's x0 before the eret
const SYS_EXIT: u64 = 0;
const SYS_PRINT: u64 = 1;
const SYS_GET_TIME: u64 = 2;
const SYS_DUMP_REGISTERS: u64 = 3;
const SYS_READ_BUTTONS: u64 = 4;
const SYS_LOAD_PROGRAM: u64 = 5;

#[unsafe(no_mangle)]
unsafe extern "C" fn _synchronous_userspace_exception(register_contents: *mut SavedRegisters) {
    // safe as this should be set from the low level handler code
    // writes through this reference end up in the program's registers after returning
    let regs = unsafe { &mut *register_contents };

    let exception_class = ESR_EL1.read(ESR_EL1::EC);
    if exception_class == 0b010101 {
        match ESR_EL1.read(ESR_EL1::ISS) {
            SYS_EXIT => {
                log!(100; "svc 0 called, exiting process\n");
            }
            SYS_PRINT => return sys_print(regs),
            SYS_GET_TIME => return sys_get_time(regs),
            SYS_DUMP_REGISTERS => return sys_dump_registers(regs),
            SYS_READ_BUTTONS => return sys_read_buttons(regs),
            SYS_LOAD_PROGRAM => return sys_load_program_from_path(regs),
            _ => {
                log!(100; "svc called with unsupported value, exiting process\n");
            }
        }
    } else {
        // not a syscall (e.g. a data abort), dump the program state to help with debugging
        // assumes uart is initialized
        uart0::write_str("synchronous userspace exception\n");
        // for a fault ELR_EL1 holds the address of the faulting instruction itself
        print_registers(regs, aarch64_cpu::registers::ELR_EL1.get());
        log!(100; "esr {:#018x}  far {:#018x}  (exception class {:#04x})\n",
            regs.esr, regs.far, exception_class);
    }

    panic!("exception from userspace, halting execution");
}

// the longest path load_program accepts
const MAX_PATH_LEN: usize = 255;

// load_program:
// takes pointer to the path in x0 and its length in x1
// on success replaces the current program with the loaded one, and doesn't return
// on failure it returns 1 in x0, logs the reason, and the caller keeps running
fn sys_load_program_from_path(regs: &mut SavedRegisters) {
    let count = regs.x1 as usize;
    if count > MAX_PATH_LEN {
        log!(100; "requested path is too long\n");
        regs.x0 = 1;
        return;
    }

    let start: *const u8 = ptr::with_exposed_provenance(regs.x0 as usize);
    let end: *const u8 = ptr::with_exposed_provenance(regs.x0 as usize + count);

    if !check_userspace_pointer(start) || !check_userspace_pointer(end) {
        log!(100; "userspace provided pointers are invalid\n");
        regs.x0 = 1;
        return;
    }

    // the path has to be copied out of userspace before loading,
    // as the program image is read over the memory it is stored in
    let mut buff: Vec<u8, MAX_PATH_LEN> = Vec::new();

    let data = slice_from_raw_parts(start, count);
    buff.extend_from_slice(unsafe { &*data }).unwrap();

    let Ok(path) = core::str::from_utf8(&buff) else {
        log!(50; "path contains invalid utf8\n");
        regs.x0 = 1;
        return;
    };

    if let Err(error) = unsafe { userspace::load_program_from_path(path) } {
        log!(300; "could not load {}: {:?}\n", path, error);
        regs.x0 = 1;
        return;
    }

    // after loading the new program, the kernel has to set up the state for an eret to enter it in el0
    unsafe { userspace::set_el0_return_state() };
}

// print:
// takes pointer to data in x0
// and length in x1
fn sys_print(regs: &SavedRegisters) {
    let count = regs.x1 as usize;
    if count > 4096 {
        log!(100; "requested print is too long\n");
        return;
    }

    let start: *const u8 = ptr::with_exposed_provenance(regs.x0 as usize);
    let end: *const u8 = ptr::with_exposed_provenance(regs.x0 as usize + count);

    if !check_userspace_pointer(start) || !check_userspace_pointer(end) {
        log!(100; "userspace provided pointers are invalid\n");
        return;
    }

    let mut buff: Vec<u8, 4096> = Vec::new();

    // slice should be safe to access as per pointer checks
    let data = slice_from_raw_parts(start, count);
    // unwrap should never fail as count is checked to be <= 4096
    buff.extend_from_slice(unsafe { &*data }).unwrap();

    let Ok(str) = String::<4096>::from_utf8(buff) else {
        log!(50;"string contains invalid utf8");
        return;
    };

    log!(4096; "{}", str);
}

// get_time:
// takes no arguments
// returns the system timer value (microseconds since boot) in x0
fn sys_get_time(regs: &mut SavedRegisters) {
    // the system timer is a 64 bit counter ticking at 1 MHz
    regs.x0 = crate::driver::timer::get_system_time();
}

// dump_registers:
// takes no arguments
// prints the program's registers as they were at the svc, modifies no registers
fn sys_dump_registers(regs: &SavedRegisters) {
    // for an svc ELR_EL1 holds the address of the instruction after it
    let svc_addr = aarch64_cpu::registers::ELR_EL1.get().wrapping_sub(4);
    print_registers(regs, svc_addr);
}

// prints the saved registers in hex, 4 per line
// pc is the address of the instruction that caused the exception
// x19-x28 are callee saved, the handler doesn't store them so they can't be printed
fn print_registers(regs: &SavedRegisters, pc: u64) {
    use core::fmt::Write;

    let gprs = [
        regs.x0, regs.x1, regs.x2, regs.x3, regs.x4, regs.x5, regs.x6, regs.x7, regs.x8, regs.x9,
        regs.x10, regs.x11, regs.x12, regs.x13, regs.x14, regs.x15, regs.x16, regs.x17, regs.x18,
    ];

    log!(20; "--- registers ---\n");
    for (row, values) in gprs.chunks(4).enumerate() {
        // 4 entries of 24 characters always fit, so the writes won't fail
        let mut line: String<128> = String::new();
        for (col, value) in values.iter().enumerate() {
            let _ = write!(line, "x{:<2} {:#018x}  ", row * 4 + col, value);
        }
        log!(128; "{}\n", line.trim_end());
    }
    log!(128; "fp  {:#018x}  lr  {:#018x}  sp  {:#018x}  pc  {:#018x}\n",
        regs.fp, regs.lr, aarch64_cpu::registers::SP_EL0.get(), pc);
}

// read_buttons:
// takes no arguments
// returns the raw levels of the button pins in x0, bit n is the level of gpio pin n (1 = high),
// bits of pins that aren't in userspace::BUTTON_PINS are always 0
// the pins are pulled up and a pressed button connects its pin to ground,
// so on real hardware pressed reads as 0 and released as 1
// in qemu the pull ups don't affect the level and qemu-gpio-input sets the level on a key press,
// so there pressed reads as 1 and released as 0
fn sys_read_buttons(regs: &mut SavedRegisters) {
    let mut levels = 0u64;
    for pin in crate::userspace::BUTTON_PINS {
        if crate::driver::gpio::get_pin_state(pin) {
            levels |= 1 << pin;
        }
    }
    regs.x0 = levels;
}

#[unsafe(no_mangle)]
unsafe extern "C" fn _unhandled_exception() -> ! {
    // assumes uart is initialized
    panic!("unhandled exception raised, kernel panic");
    //returning from this is UB as it will clobber registers
}
