set quiet := true
set windows-shell := ["nu", "-c"]

bsp := "rpi4"

target := "aarch64-unknown-none-softfloat"
target_path := "target/" + target + "/release"

kernel_elf := target_path + "/kernel"
kernel_img := target_path + "/kernel8.img"

export RUSTFLAGS := "-C link-arg=--library-path=src/bsp/rpi -C link-arg=--script=kernel.ld"

qemu_machine := if bsp == "rpi4" {
    "raspi4b"
} else {
    error("unknown bsp: " + bsp)
}

default:
    just --list

build:
    cargo rustc --features=bsp_{{bsp}} --target {{target}} --release
    rust-objcopy {{kernel_elf}} --strip-all -O binary {{kernel_img}}

run: build
    qemu-system-aarch64 -M {{qemu_machine}} -d in_asm -display none -kernel {{kernel_img}}
