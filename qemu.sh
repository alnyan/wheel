#!/bin/sh

set -e

mkdir -p build

if [ ! -d "build/yboot2" ]; then
    git clone https://git.alnyan.me/yggdrasil/yboot2.git build/yboot2
fi

cargo build
pushd build/yboot2
cargo build
popd

IMAGE="build/image.fat32"

dd if=/dev/zero of=${IMAGE} bs=1M count=64
mkfs.vfat -F32 ${IMAGE}
mcopy -i ${IMAGE} build/yboot2/target/x86_64-unknown-uefi/debug/yboot2.efi ::yboot2.efi
mcopy -i ${IMAGE} target/x86_64-unknown-none/debug/osdev-amd64 ::kernel.elf

qemu-system-x86_64 \
    -drive readonly,format=raw,if=pflash,file=/usr/share/edk2-ovmf/OVMF_CODE.fd \
    -drive format=raw,file=${IMAGE} \
    -net none \
    -s \
    -serial mon:stdio \
    -enable-kvm \
    -cpu host
