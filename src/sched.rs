use crate::proc::Process;
use alloc::boxed::Box;
use core::ptr::null_mut;
use spin::Mutex;

// Needs manual locking because second "switch" will
// result in deadlock if usual locking on whole scheduler
// struct
struct Scheduler {
    current: Mutex<*mut Process>,
    idle: *mut Process,
    queue_head: Mutex<*mut Process>,
}

unsafe impl Send for Scheduler {}

pub struct SchedulerHead {
    prev: *mut Process,
    next: *mut Process,
}

static mut CPU0: Scheduler = Scheduler {
    current: Mutex::new(null_mut()),
    idle: null_mut(),
    queue_head: Mutex::new(null_mut()),
};

impl SchedulerHead {
    pub const fn empty() -> SchedulerHead {
        SchedulerHead {
            prev: null_mut(),
            next: null_mut(),
        }
    }
}

impl Scheduler {
    pub fn queue(&mut self, proc: &mut Process) {
        let mut queue_lock = self.queue_head.lock();

        if queue_lock.is_null() {
            *queue_lock = proc;
            proc.sched = SchedulerHead {
                prev: proc,
                next: proc,
            }
        } else {
            unsafe {
                let tail = (**queue_lock).sched.prev;

                (*tail).sched.next = proc;
                (*proc).sched.prev = tail;
                (**queue_lock).sched.prev = proc;
                (*proc).sched.next = *queue_lock;
            }
        }
    }

    pub fn switch(&mut self) {
        // TODO: lock once on "current", release right before switching
        let next: *mut Process;
        let curr = *self.current.lock();
        {
            let queue_lock = self.queue_head.lock();

            if !curr.is_null() && unsafe { !(*curr).sched.next.is_null() } {
                next = unsafe { (*curr).sched.next };
            } else if !queue_lock.is_null() {
                next = *queue_lock;
            } else {
                next = self.idle;
            }
        }

        assert!(!next.is_null());
        if next != curr {
            *self.current.lock() = next;
            unsafe { (*curr).context.switch_to(&mut (*next).context) };
        }
    }
}

pub fn r#yield() {
    unsafe { CPU0.switch() };
}

/// # Safety
///
/// The function must only be called ONCE and AFTER init() has been called
pub unsafe fn enter() -> ! {
    let proc: *mut Process;
    {
        let queue_lock = CPU0.queue_head.lock();
        if queue_lock.is_null() {
            proc = CPU0.idle;
        } else {
            proc = *queue_lock;
        }

        *CPU0.current.lock() = proc;
    }
    (*proc).context.initial_switch();

    panic!("Didn't enter the thread");
}

// XXX: This hack ignores the concept of lifetimes
//      for ease of access
pub fn current_proc() -> &'static mut Process {
    unsafe { &mut **CPU0.current.lock() }
}

pub fn queue(proc: &mut Process) {
    unsafe { CPU0.queue(proc) };
}

pub fn idle(_: usize) {
    loop {
        unsafe {
            llvm_asm!("hlt");
        }
    }
}

pub fn init() {
    let idle = Box::new(Process::kspawn(idle, 0, "idle"));
    unsafe {
        CPU0.idle = Box::into_raw(idle);
    }
}
