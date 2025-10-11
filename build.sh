#!/bin/bash
RUSTFLAGS="-C link-arg=--library-path=src/bsp/rpi -C link-arg=--script=kernel.ld " cargo rustc --features=bsp_rpi4 --target aarch64-unknown-none-softfloat --release &&
  rust-objcopy target/aarch64-unknown-none-softfloat/release/kernel --strip-all -O binary target/aarch64-unknown-none-softfloat/release/kernel8.img &&
  qemu-system-aarch64 -M raspi4b -d in_asm -display none -kernel target/aarch64-unknown-none-softfloat/release/kernel8.img
