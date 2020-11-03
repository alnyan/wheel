
pub fn init() {
    use crate::arch::x86;
    x86::syscall::init();
}
