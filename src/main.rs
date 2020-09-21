#![feature(llvm_asm, global_asm, const_in_array_repeat_expressions)]

#![no_main]
#![no_std]

extern crate yboot2_proto;

#[macro_use]
pub mod debug;
pub mod dev;
pub mod arch;
pub mod mem;
mod boot;

#[no_mangle]
pub extern "C" fn kernel_main() {
    // Cleanup terminal colors after UEFI
    print!("\x1b[0m\x1b[2J\x1B[0;0f");
    let boot = boot::boot_data();
    use yboot2_proto::Magic;
    assert!(boot.hdr.loader_magic == yboot2_proto::ProtoV1::LOADER_MAGIC);

    arch::x86::gdt::init();
    arch::x86::idt::init();

    mem::phys::init(&boot.memory_map);

    println!("Survived");

    loop {}
}

#[panic_handler]
fn panic_handler(_pi: &core::panic::PanicInfo) -> ! {
    loop {}
}
