set quiet := true
set windows-shell := ["nu", "-c"]

target := "aarch64-unknown-none-softfloat"
target_path := "target/" + target + "/release"

boot_elf := target_path + "/boot"
boot_bin := target_path + "/boot.bin"

kernel_elf := target_path + "/kernel"
kernel_bin := target_path + "/kernel.bin"

kernel8 := target_path + "/kernel8.img"

boot_rustflags := "-C link-arg=--script=boot/boot.ld"
kernel_rustflags := "-C link-arg=--script=kernel/kernel.ld -C target-feature=+neon"

default:
    just --list

build:
    RUSTFLAGS="{{boot_rustflags}}" cargo rustc -p boot --release --target {{target}}
    rust-objcopy {{boot_elf}} --strip-all -O binary {{boot_bin}}
    RUSTFLAGS="{{kernel_rustflags}}" cargo rustc -p kernel --release --target {{target}}
    rust-objcopy {{kernel_elf}} --strip-all -O binary {{kernel_bin}}
    cargo run -p mk-kernel8 {{boot_bin}} {{kernel_bin}} {{kernel8}}

[parallel]
run: qemu-run qemu-gpio-input

qemu-gpio-input:
    cargo run -p qemu-gpio-input

qemu-run: build
    qemu-system-aarch64 -M raspi4b -serial mon:stdio -kernel {{kernel8}} -qtest tcp:127.0.0.1:5000,server,nowait

asm: build
    qemu-system-aarch64 -d in_asm -M raspi4b -kernel {{kernel8}}

monitor: build
    qemu-system-aarch64 -monitor stdio -M raspi4b -kernel {{kernel8}}
