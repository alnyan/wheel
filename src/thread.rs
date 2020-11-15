pub use crate::arch::x86::context::Context;
use alloc::boxed::Box;
use core::ptr::null_mut;

pub struct Process {
    pub id: i32,
    pub is_user: bool,
    pub head: *mut Thread
}

pub struct Thread {
    pub context: Context,

    pub owner: *mut Process,

    pub thread_prev: *mut Thread,
    pub thread_next: *mut Thread,

    pub sched_prev: *mut Thread,
    pub sched_next: *mut Thread,
}

pub struct Stats {
    pub idle_ticks: u64,
    pub total_ticks: u64
}

impl Process {
    pub fn new_kernel() -> Process {
        println!("Create new empty process");
        Process {
            id: 0,  // TODO
            is_user: false,
            head: null_mut(),
        }
    }

    pub fn spawn(&mut self, entry: usize, arg: usize) -> Option<*mut Thread> {
        println!("Spawn a thread in process #{}", self.id);

        // TODO: user threads
        let thread = Box::into_raw(Box::new(Thread::new_kernel(self as *mut Process, entry, arg)));
        unsafe { (*thread).thread_next = self.head; }
        self.head = thread;
        unsafe { (*thread).queue(); }

        Some(thread)
    }
}

impl Thread {
    fn new_kernel(owner: *mut Process,
                  entry: usize,
                  _arg: usize) -> Thread {
        Thread {
            context: Context::new(entry),

            owner,

            thread_next: null_mut(),
            thread_prev: null_mut(),

            sched_prev: null_mut(),
            sched_next: null_mut(),
        }
    }

    pub fn is_current(&self) -> bool {
        return unsafe { self as *const Thread == CURRENT };
    }

    pub fn current() -> *mut Thread {
        unsafe {
            assert!(!CURRENT.is_null());
            return CURRENT;
        }
    }

    pub fn terminate(&mut self) {
        self.dequeue();
    }

    fn queue(&mut self) {
        unsafe { llvm_asm!("cli"); }
        assert!(self.sched_prev.is_null() == self.sched_next.is_null());

        if self.sched_prev.is_null() {
            unsafe {
                if QUEUE_HEAD.is_null() {
                    self.sched_prev = self;
                    self.sched_next = self;

                    QUEUE_HEAD = self;
                } else {
                    let tail = (*QUEUE_HEAD).sched_prev;

                    (*tail).sched_next = self;
                    self.sched_prev = tail;
                    (*QUEUE_HEAD).sched_prev = self;
                    self.sched_next = QUEUE_HEAD;
                }
            }
        }
    }

    fn dequeue(&mut self) {
        unsafe { llvm_asm!("cli"); }
        assert!(self.sched_prev.is_null() == self.sched_next.is_null());

        let prev = self.sched_prev;
        let next = self.sched_next;

        if prev.is_null() {
            return;
        }

        unsafe {
            if next == self as *mut Thread {
                self.sched_prev = null_mut();
                self.sched_next = null_mut();

                QUEUE_HEAD = null_mut();

                CURRENT = IDLE;
                self.context.switch_to(&mut (*IDLE).context);
            }

            if QUEUE_HEAD == self as *mut Thread {
                QUEUE_HEAD = next;
            }

            (*prev).sched_next = next;
            (*next).sched_prev = prev;

            self.sched_next = null_mut();
            self.sched_prev = null_mut();

            CURRENT = next;
            self.context.switch_to(&mut (*next).context);
        }
    }
}

static mut QUEUE_HEAD: *mut Thread = null_mut();
#[no_mangle]
pub static mut CURRENT: *mut Thread = null_mut();
static mut IDLE: *mut Thread = null_mut();
static mut STATS: Stats = Stats {
    idle_ticks: 0,
    total_ticks: 0
};

pub fn idle(_: usize) {
    loop {
        unsafe { llvm_asm!("hlt"); }
    }
}

pub fn setup() {
    let idle = Box::new(Thread::new_kernel(null_mut(), idle as usize, 0));
    unsafe {
        IDLE = Box::into_raw(idle);
    }
}

pub unsafe fn enter() -> ! {
    assert!(!IDLE.is_null());

    if QUEUE_HEAD.is_null() {
        CURRENT = IDLE;
    } else {
        CURRENT = QUEUE_HEAD;
    }

    (*CURRENT).context.initial_switch();
    panic!("Didn't enter the thread");
}

pub unsafe fn r#yield() {
    let next: *mut Thread;
    let curr = CURRENT;

    STATS.total_ticks += 1;
    if curr == IDLE {
        STATS.idle_ticks += 1;
    }

    if STATS.total_ticks >= 400 {
        println!("Load: {}%", (STATS.total_ticks - STATS.idle_ticks) / (STATS.total_ticks / 100));
        STATS.total_ticks = 0;
        STATS.idle_ticks = 0;
    }

    if !curr.is_null() && !(*curr).sched_next.is_null() {
        next = (*curr).sched_next;
    } else if !QUEUE_HEAD.is_null() {
        next = QUEUE_HEAD;
    } else {
        next = IDLE;
    }

    assert!(!next.is_null());
    CURRENT = next;

    (*curr).context.switch_to(&mut (*next).context);
}
