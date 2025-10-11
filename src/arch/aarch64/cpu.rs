pub fn spin_forever() -> ! {
    loop {
        aarch64_cpu::asm::wfe();
    }
}
