#[no_mangle]
pub static mut SYSCALL_TABLE: [usize; 256] = [0; 256];

macro_rules! sys_set {
    ($n:expr, $handler:expr) => {
        SYSCALL_TABLE[$n] = $handler as usize
    }
}

fn sys_test() {
}

pub fn init() {
    // Initialize syscall "vectors"
    unsafe {
        sys_set!(1, sys_test);
    }

    // Platform-specific init
    use crate::arch::x86;
    x86::syscall::init();
}
