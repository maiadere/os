use aarch64_cpu::asm;

pub fn spin_forever() -> ! {
    loop {
        asm::wfe();
    }
}
