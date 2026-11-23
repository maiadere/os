set quiet := true
set windows-shell := ["nu", "-c"]

target := "aarch64-unknown-none-softfloat"
target_path := "target/" + target + "/release"

boot_elf := target_path + "/boot"
boot_bin := target_path + "/boot.bin"

kernel_elf := target_path + "/kernel"
kernel_bin := target_path + "/kernel.bin"

kernel8 := target_path + "/kernel8.img"

# the pristine card image tracked in git, and the one the kernel embeds with programs added
sd_base := "kernel/assets/sd.img.base"
sd_img := "kernel/assets/sd.img"
sd_img_new := "kernel/assets/sd.img.new"
# byte offset of the fat32 partition in the image, lba 2048 * 512 bytes per sector
fat_offset := "1048576"

boot_rustflags := "-C link-arg=--script=boot/boot.ld"
kernel_rustflags := "-C link-arg=--script=kernel/kernel.ld -C target-feature=+neon"
program_rustflags := "-C relocation-model=static"

default:
    just --list

# builds one of the crates under programs/ into a flat image the kernel can load as is
program name:
    RUSTFLAGS="{{program_rustflags}} -C link-arg=--script=programs/{{name}}/program.ld" cargo rustc -p {{name}} --release --target {{target}}
    rust-objcopy {{target_path}}/{{name}} --strip-all -O binary {{target_path}}/{{name}}.bin

# rebuilds the card image from the pristine one with the userspace programs copied onto it
# the kernel embeds this image with include_bytes!, so it has to be up to date before it builds
# add a program by building it above and giving it another mcopy line here
sdimg: (program "hello")
    cp {{sd_base}} {{sd_img_new}}
    mcopy -o -i {{sd_img_new}}@@{{fat_offset}} {{target_path}}/hello.bin ::/HELLO.BIN
    cmp -s {{sd_img_new}} {{sd_img}} || mv -f {{sd_img_new}} {{sd_img}}
    rm -f {{sd_img_new}}
    mdir -i {{sd_img}}@@{{fat_offset}} ::

build *flags: sdimg
    RUSTFLAGS="{{boot_rustflags}}" cargo rustc -p boot --release --target {{target}}
    rust-objcopy {{boot_elf}} --strip-all -O binary {{boot_bin}}
    RUSTFLAGS="{{kernel_rustflags}}" cargo rustc -p kernel --release --target {{target}} {{flags}}
    rust-objcopy {{kernel_elf}} --strip-all -O binary {{kernel_bin}}
    cargo run -p mk-kernel8 {{boot_bin}} {{kernel_bin}} {{kernel8}}

run: (build "--features qemu")
    qemu-system-aarch64 -M raspi4b -serial mon:stdio -kernel {{kernel8}}

asm: (build "--features qemu")
    qemu-system-aarch64 -d in_asm -M raspi4b -kernel {{kernel8}}

monitor: (build "--features qemu")
    qemu-system-aarch64 -monitor stdio -M raspi4b -kernel {{kernel8}}

sdspi: build
    qemu-system-aarch64 -serial mon:stdio -M raspi4b -kernel {{kernel8}} -device sd-card-spi,drive=spisdcard -drive file=sdcard.img,format=raw,id=spisdcard,if=none 
# -trace "bcm2835*"
