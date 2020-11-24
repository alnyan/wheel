pub use crate::arch::x86::context::Context;
use crate::sched;

pub struct Process {
    pub id: i32,

    pub context: Context,
    pub sched: sched::SchedulerHead,
}

impl Process {
    pub fn kspawn(func: fn (usize) -> (), _arg: usize, name: &str) -> Process {
        let pid = unsafe {
            PID_COUNTER += 1;
            PID_COUNTER
        };
        println!("kspawn(\"{}\") -> {}", name, pid);

        Process {
            id: pid,
            context: Context::new(func as usize),
            sched: sched::SchedulerHead::empty()
        }
    }

    pub fn current() -> &'static mut Process {
        sched::current_proc()
    }
}

static mut PID_COUNTER: i32 = 0;
