#![allow(clippy::new_without_default)]

pub struct IrqDisable {
    saved_rflags: u64
}

impl IrqDisable {
    pub fn new() -> IrqDisable {
        let mut rflags: u64;
        unsafe { llvm_asm!("pushfq; pop $0; cli":"=r"(rflags)::"memory"); }
        IrqDisable {
            saved_rflags: rflags
        }
    }
}

impl Drop for IrqDisable {
    fn drop(&mut self) {
        if self.saved_rflags & (1 << 9) != 0 {
            unsafe { llvm_asm!("sti"); }
        }
    }
}
