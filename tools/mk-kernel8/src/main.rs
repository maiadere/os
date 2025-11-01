const BOOT_CODE_MAX_SIZE: usize = 0x1000;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() != 4 {
        println!("Usage: {} <boot.bin> <kernel.bin> <kernel8.img>", args[0]);
        return;
    }

    let boot = std::fs::read(&args[1]).expect(&format!("failed to read {}", args[1]));
    let kernel = std::fs::read(&args[2]).expect(&format!("failed to read {}", args[2]));

    if boot.len() > BOOT_CODE_MAX_SIZE {
        panic!(
            "boot code size ({}) exceeded max size ({})",
            boot.len(),
            BOOT_CODE_MAX_SIZE
        );
    }

    let mut kernel8 = boot;
    kernel8.resize(BOOT_CODE_MAX_SIZE, 0);
    kernel8.extend(kernel.into_iter());

    std::fs::write(&args[3], kernel8).expect(&format!("failed to write {}", args[3]));
}
