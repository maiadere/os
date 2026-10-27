#![allow(dead_code)]
#![no_main]
#![no_std]

mod console;
mod driver;
mod exception;
mod fat32;
mod mmio;
mod userspace;

use aarch64_cpu::{
    asm,
    registers::{self, Readable, VBAR_EL1},
};
use core::{arch::asm, ops::ControlFlow, panic::PanicInfo};
use driver::{uart0, videocore::framebuffer::Framebuffer};
use heapless::format;

use crate::{
    driver::{
        sd_spi::{self, parse_mbr},
        spi,
        videocore::mailbox::{
            PropertyTag, mailbox_property_send,
            property::{
                GET_ARM_MEMORY, GET_BOARD_MAC_ADDRESS, GET_BOARD_MODEL, GET_BOARD_SERIAL,
                GET_CLOCK_RATE_MEASURED, GET_VC_MEMORY,
            },
        },
    },
    fat32::{DirEntry, Fat32, find_partition_start},
};

fn kernel_main() -> ! {
    // post boot init
    uart0::init();
    unsafe {
        exception::set_vbar_el1();
        memory::mmu::post_boot_mappings();
    }

    // let fb = Framebuffer::init(1024, 600, 32, 1).unwrap();
    //
    // log!(750; "{}\n", include_str!("../assets/logo.txt"));

    // at this point the exceptions, uart printing and virtual memory are set up
    log!(100; "Running kernel_main at address {:p}\n", kernel_main as *const ());
    log!(20; "Current EL: {}\n", ((&registers::CurrentEL).get()) >> 2);
    log!(100; "Address in vbar_el1: {:p}\n", VBAR_EL1.get() as *const ());
    log!(100; "Current stack pointer: {:p}\n", registers::SP.get() as *const ());
    // log!(100; "Framebuffer address: {:p}\n", fb.addr as *const ());

    log!(100; "Board model: {}\n", mailbox_property_send(PropertyTag::new(GET_BOARD_MODEL, 0u32))
        .unwrap()
        .value);
    log!(100; "Board serial: {}\n", mailbox_property_send(PropertyTag::new(GET_BOARD_SERIAL, 0u64))
        .unwrap()
        .value);

    let mac = mailbox_property_send(PropertyTag::new(GET_BOARD_MAC_ADDRESS, [0u8; 6]))
        .unwrap()
        .value;
    log!(100; "MAC Address: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}\n", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);

    let arm_mem = mailbox_property_send(PropertyTag::new(GET_ARM_MEMORY, (0u32, 0u32)))
        .unwrap()
        .value;
    log!(100; "ARM memory base address: {:p}\n", arm_mem.0 as *const ());
    log!(100; "ARM memory size: {}MiB\n", arm_mem.1 / (1024 * 1024));

    let vc_mem = mailbox_property_send(PropertyTag::new(GET_VC_MEMORY, (0u32, 0u32)))
        .unwrap()
        .value;
    log!(100; "VideoCore memory base address: {:p}\n", vc_mem.0 as *const ());
    log!(100; "VideoCore memory size: {}MiB\n", vc_mem.1 / (1024 * 1024));

    let clock_rate = mailbox_property_send(PropertyTag::new(GET_CLOCK_RATE_MEASURED, (3u32, 0u32)))
        .unwrap()
        .value;
    log!(100; "Measured clock rate: {:.2}GHz\n\n", clock_rate.1 as f64 / 1e9);

    // console::render(&fb);

    unsafe { spi::initialize_spi0() };
    unsafe { sd_spi::initalize_sd_spi().unwrap() };
    let start = find_partition_start().unwrap();
    log!(100; "found fat partition at: {}\n", &start);
    let fat = Fat32::mount(start).unwrap();
    print_tree(&fat, &fat.root(), 0);
    // let mbr = sd_spi::read_sd_block(0).unwrap();
    // let partition_table = parse_mbr(&mbr);
    // log!(1024;"{:?}\n", partition_table);

    // let partition_bytes = &mbr[0x1be..];
    //
    // for (idx, byte) in mbr.iter().enumerate() {
    //     log!(3; "{:2x} ", *byte);
    //     if idx % 32 == 0 && idx != 0 {
    //         log!(1;"\n");
    //     }
    // }
    // log!(1;"\n");
    panic!("kernel over");
    // let test_code: [u8; _] = [
    //     0x1f, 0x20, 0x03, 0xd5, //nop
    //     0x21, 0x00, 0x00, 0xd4, //svc #1
    //     0x41, 0x00, 0x00, 0xd4, //svc #2
    //     0x01, 0x00, 0x00, 0xd4, //svc #0
    // ];
    // let test_code: [u8; _] = [
    //     0xa0, 0x00, 0x00, 0x10, //
    //     0x81, 0x01, 0x80, 0xd2, //
    //     0x21, 0x00, 0x00, 0xd4, //
    //     0x00, 0x00, 0x00, 0x10, //
    //     0x01, 0x00, 0x00, 0xd4, //
    //     0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21,
    // ];
    // unsafe { userspace::load_program(&test_code) };
    // let test = driver::sdhci::muxing_state();
    // log!(100;"muxing state: {:b}\n", test);
    // let test = driver::sdhci::test_capabilities();
    // log!(100;"sdhci capabilities: {:b}\n", test);
    // let test = driver::sdhci::test_presence();
    // log!(100;"sdhci presence: {:b}\n", test);
    // panic!("kernel over");
}
fn print_tree(fs: &Fat32, dir: &DirEntry, depth: usize) {
    fs.read_dir(dir, |x| {
        for _ in 0..depth {
            log!(2; "  ");
        }
        log!(1000; "{}\n",x.name());
        if x.is_dir && x.name() != "." && x.name() != ".." {
            print_tree(fs, x, depth + 1);
        }
        ControlFlow::<()>::Continue(())
    });
}

#[unsafe(no_mangle)]
pub unsafe fn _start_rust() -> ! {
    unsafe {
        asm!(
            ".macro ADR_REL register, symbol",
            "    adrp \\register, \\symbol",
            "    add \\register, \\register, #:lo12:\\symbol",
            ".endm",
            "    ADR_REL x0, __bss_start",
            "    ADR_REL x1, __bss_end",
            /* zero the bss segment */
            "0:",
            "    cmp x0, x1",
            "    b.eq 1f",
            "    stp xzr, xzr, [x0], #16",
            "    b 0b",
            "1:",
        );
    }
    kernel_main()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(panic_message) = format!(2048;"{}\n", info).ok() {
        uart0::write_str(panic_message.as_str());
    } else {
        uart0::write_str("Kernel panic; panic message too long!");
    }
    loop {
        asm::wfe();
    }
}
