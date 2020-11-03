#!/bin/sh

set -e

IMAGE="build/image.fat32"
CARGO_ARGS=""
QEMU_ARGS=""
NAME="wheel"
KERNEL="target/x86_64-unknown-none/debug/$NAME"

if [ "$QEMU_DEBUG" = 1 ]; then
    QEMU_ARGS="$QEMU_ARGS -S"
fi

if [ ! "$QEMU_KVM" = 0 ]; then
    QEMU_ARGS="$QEMU_ARGS -enable-kvm -cpu host"
fi

if [ "$RELEASE" = 1 ]; then
    CARGO_ARGS="$CARGO_ARGS --release"
    KERNEL="target/x86_64-unknown-none/release/$NAME"
fi

mkdir -p build

if [ ! -d "build/yboot2" ]; then
    git clone https://git.alnyan.me/yggdrasil/yboot2.git build/yboot2
fi

cargo build $CARGO_ARGS
pushd build/yboot2
cargo build
popd

dd if=/dev/zero of=${IMAGE} bs=1M count=64
mkfs.vfat -F32 ${IMAGE}
mmd -i ${IMAGE} ::EFI
mmd -i ${IMAGE} ::EFI/Boot
mcopy -i ${IMAGE} build/yboot2/target/x86_64-unknown-uefi/debug/yboot2.efi ::EFI/Boot/bootx64.efi
mcopy -i ${IMAGE} ${KERNEL} ::kernel.elf

qemu-system-x86_64 \
    -drive readonly,format=raw,if=pflash,file=/usr/share/edk2-ovmf/OVMF_CODE.fd \
    -drive format=raw,file=${IMAGE} \
    -net none \
    -s \
    -serial mon:stdio \
    $QEMU_ARGS
