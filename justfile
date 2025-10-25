set quiet := true
set windows-shell := ["nu", "-c"]

target := "aarch64-unknown-none-softfloat"
target_path := "target/" + target + "/release"

kernel_elf := target_path + "/kernel"
kernel_img := target_path + "/kernel8.img"

export RUSTFLAGS := "-C link-arg=--library-path=src/boot -C link-arg=--script=kernel.ld"

default:
    just --list

build:
    cargo rustc --release
    rust-objcopy {{kernel_elf}} --strip-all -O binary {{kernel_img}}

run: build
    qemu-system-aarch64 -M raspi4b -serial mon:stdio -kernel {{kernel_img}}
