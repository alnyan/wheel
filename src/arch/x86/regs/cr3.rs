#[inline(always)]
pub fn read() -> usize {
    let mut val: usize;
    unsafe { llvm_asm!("mov %cr3, $0":"=r"(val)) }
    val
}

#[inline(always)]
pub unsafe fn write(value: usize) {
    llvm_asm!("mov $0, %cr3"::"r"(value):"memory");
}
