#![feature(llvm_asm, global_asm)]

#![no_main]
#![no_std]

extern crate yboot2_proto;

#[macro_use]
pub mod debug;
pub mod dev;
mod boot;

#[no_mangle]
pub extern "C" fn kernel_main() {
    println!("Hello");
    loop {}
}

#[panic_handler]
fn panic_handler(_pi: &core::panic::PanicInfo) -> ! {
    loop {}
}
