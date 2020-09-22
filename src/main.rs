#![feature(llvm_asm, global_asm, const_in_array_repeat_expressions, const_fn, array_map)]

#![no_main]
#![no_std]

extern crate yboot2_proto;

pub const KERNEL_OFFSET: usize = 0xFFFFFF0000000000;

#[inline(always)]
pub fn virtualize(phys: usize) -> usize {
    assert!(phys < 0x100000000);    // Because above 4GiB isn't mapped yet
    phys + KERNEL_OFFSET
}

#[macro_use]
pub mod debug;
pub mod dev;
pub mod arch;
pub mod mem;
pub mod sync;
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

    // Initialize local APIC
    dev::x86::apic::init(virtualize(0xFEE00000));
    dev::x86::acpi::init(Some(boot.rsdp as usize));
    dev::x86::ps2::init();

    info!("Test info\n");
    warn!("Test warning\n");
    error!("Test error\n");
    fatal!("Test fatal\n");

    println!("Survived");

    loop {
        unsafe { llvm_asm!("sti; hlt"); }
    }
}

#[panic_handler]
fn panic_handler(pi: &core::panic::PanicInfo) -> ! {
    println!("{}", pi);
    loop {}
}
