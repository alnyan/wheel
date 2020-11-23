#[inline(always)]
pub fn read() -> usize {
    let mut val: usize;
    unsafe { llvm_asm!("mov %cr2, $0":"=r"(val)) }
    val
}
