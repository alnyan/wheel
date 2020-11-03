pub use crate::arch::x86::context::Context;

pub struct Thread {
    pub context: Context,
    pub next: *mut Thread,
}

impl Thread {
    pub fn new(entry: fn(usize) -> (), _arg: usize) -> Thread {
        Thread {
            context: Context::new(entry as usize),
            next: core::ptr::null_mut()
        }
    }

    pub fn next(&self) -> &'static mut Thread {
        return unsafe { self.next.as_mut() }.unwrap();
    }
}

static mut QUEUE_HEAD: *mut Thread = core::ptr::null_mut();
#[no_mangle]
pub static mut CURRENT: *mut Thread = core::ptr::null_mut();

pub unsafe fn enter() -> ! {
    assert!(!QUEUE_HEAD.is_null());
    CURRENT = QUEUE_HEAD;
    (*CURRENT).context.initial_switch();
    loop {}
}

pub unsafe fn enqueue(thread: *mut Thread) {
    (*thread).next = QUEUE_HEAD;
    QUEUE_HEAD = thread;
}

pub unsafe fn r#yield() {
    assert!(!CURRENT.is_null());
    assert!(!QUEUE_HEAD.is_null());

    let prev = CURRENT;
    let mut next = (*prev).next;

    if next.is_null() {
        next = QUEUE_HEAD;
    }

    CURRENT = next;
    (*prev).context.switch_to(&mut (*next).context);
}
