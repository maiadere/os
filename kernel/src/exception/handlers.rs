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

#[unsafe(no_mangle)]
unsafe extern "C" fn _synchronous_userspace_exception(register_contents: *mut SavedRegisters) {
    // assumes uart is initialized
    uart0::write_str("synchronous userspace exception\n");
    // safe as this should be set from the low level handler code
    let register_contents = unsafe { *register_contents };
    uart0::write_str(format!(512; "{:?}\n", register_contents).unwrap().as_str());

    let exception_class = ESR_EL1.read(ESR_EL1::EC);
    // log!(30; "exception class: {}\n", exceptio);
    if exception_class == 0b010101 {
        let syscall_num = ESR_EL1.read(ESR_EL1::ISS);
        match syscall_num {
            0 => {
                log!(100; "svc 0 called, exiting process\n");
            }
            // print:
            // takes pointer to data in x0
            // and length in x1
            1 => {
                log!(100; "svc 1 called\n");

                let count = register_contents.x1 as usize;
                if count > 4096 {
                    log!(100; "requested print is too long\n");
                    return;
                }

                let start: *const u8 = ptr::with_exposed_provenance(register_contents.x0 as usize);
                let end: *const u8 =
                    ptr::with_exposed_provenance(register_contents.x0 as usize + count);

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

                return;
            }
            2 => {
                log!(100; "svc 2 called\n");
                return;
            }
            _ => {
                log!(100; "svc called with unsupported value, exiting process\n");
            }
        }
    }

    panic!("exception from userspace, halting execution");
}

#[unsafe(no_mangle)]
unsafe extern "C" fn _unhandled_exception() -> ! {
    // assumes uart is initialized
    panic!("unhandled exception raised, kernel panic");
    //returning from this is UB as it will clobber registers
}
