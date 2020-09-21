#![feature(llvm_asm, global_asm, const_in_array_repeat_expressions)]

#![no_main]
#![no_std]

extern crate yboot2_proto;

#[macro_use]
pub mod debug;
pub mod dev;
pub mod arch;
mod boot;

#[no_mangle]
pub extern "C" fn kernel_main() {
    // Cleanup terminal colors after UEFI
    print!("\x1b[0m");

    arch::x86::gdt::init();
    arch::x86::idt::init();

    let addr = (4 * 1024 * 1024 * 1024usize);
    unsafe {
        let mut ptr = addr as *mut u32;

        *ptr = 1234;
    }

    println!("Survived");

    loop {}
}

#[panic_handler]
fn panic_handler(_pi: &core::panic::PanicInfo) -> ! {
    loop {}
}
