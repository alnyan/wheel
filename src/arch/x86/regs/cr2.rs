#[inline(always)]
pub fn read() -> usize {
    let mut val: usize;
    unsafe { llvm_asm!("mov %cr2, $0":"=r"(val)) }
    val
}

#[inline(always)]
pub unsafe fn write(value: usize) {
    llvm_asm!("mov $0, %cr2"::"r"(value):"memory");
}
