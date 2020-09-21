pub fn halt() -> ! {
    loop {
        unsafe { llvm_asm!("cli; hlt"); }
    }
}
