set quiet := true
set windows-shell := ["nu", "-c"]

target := "aarch64-unknown-none-softfloat"
target_path := "target/" + target + "/release"

boot_elf := target_path + "/boot"
boot_bin := target_path + "/boot.bin"

kernel_elf := target_path + "/kernel"
kernel_bin := target_path + "/kernel.bin"

kernel8 := target_path + "/kernel8.img"

shell_elf := target_path + "/shell"

boot_rustflags := "-C link-arg=--script=boot/boot.ld"
kernel_rustflags := "-C link-arg=--script=kernel/kernel.ld -C target-feature=+neon"
shell_rustflags := "-C link-arg=--script=shell/shell.ld"

default:
    just --list

build *flags:
    RUSTFLAGS="{{boot_rustflags}}" cargo rustc -p boot --release --target {{target}} {{flags}}
    rust-objcopy {{boot_elf}} --strip-all -O binary {{boot_bin}}
    RUSTFLAGS="{{shell_rustflags}}" cargo rustc -p shell --release --target {{target}}
    rust-objcopy {{shell_elf}} --strip-all -O binary sd/system/shell
    RUSTFLAGS="{{kernel_rustflags}}" cargo rustc -p kernel --release --target {{target}} {{flags}}
    rust-objcopy {{kernel_elf}} --strip-all -O binary {{kernel_bin}}
    cargo run -p mk-kernel8 {{boot_bin}} {{kernel_bin}} {{kernel8}}

run: (build "--features qemu")
    qemu-system-aarch64 -M raspi4b -serial mon:stdio -kernel {{kernel8}}

[parallel]
run-wayland: qemu-wayland qemu-gpio-input

qemu-wayland: (build "--features qemu")
    GDK_BACKEND=x11 qemu-system-aarch64 -M raspi4b -display gtk -serial mon:stdio -kernel {{kernel8}} -qtest tcp:127.0.0.1:5000,server,nowait

qemu-gpio-input:
    -cargo run -p qemu-gpio-input

asm: (build "--features qemu")
    qemu-system-aarch64 -d in_asm -M raspi4b -kernel {{kernel8}}

monitor: (build "--features qemu")
    qemu-system-aarch64 -monitor stdio -M raspi4b -kernel {{kernel8}}

sdspi: build
    qemu-system-aarch64 -serial mon:stdio -M raspi4b -kernel {{kernel8}} -device sd-card-spi,drive=spisdcard -drive file=sdcard.img,format=raw,id=spisdcard,if=none 
# -trace "bcm2835*"
