pub use crate::arch::x86::context::Context;

pub fn idle(_: usize) {
    loop {
        unsafe {
            llvm_asm!("hlt");
        }
    }
}

pub fn setup() {
    //let idle = Box::new(Thread::new_kernel(null_mut(), idle as usize, 0));
    //unsafe {
    //    IDLE = Box::into_raw(idle);
    //}
}

pub unsafe fn enter() -> ! {
    panic!("Didn't enter the thread");
}

pub unsafe fn r#yield() {
}
