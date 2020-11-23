pub trait IrqHandler {
    fn handle(&mut self) -> bool;
}

struct IrqHandlerBind {
    #[allow(dead_code)]
    data: u128
}

impl IrqHandlerBind {
    pub const fn new(h: &'static mut dyn IrqHandler) -> IrqHandlerBind {
        IrqHandlerBind {
            data: unsafe {core::mem::transmute::<_, u128>(h)}
        }
    }

    pub fn invoke(&mut self) -> bool {
        let h: &'static mut dyn IrqHandler = unsafe {core::mem::transmute(self.data)};
        h.handle()
    }
}

struct IrqVector {
    handlers: [Option<IrqHandlerBind>; MAX_SLOT],
}

impl IrqVector {
    pub const fn empty() -> IrqVector {
        IrqVector {
            handlers: [None; MAX_SLOT]
        }
    }

    pub fn add_slot(&mut self, h: IrqHandlerBind) -> Option<SlotNumber> {
        for i in 0 .. MAX_SLOT {
            if self.handlers[i].is_none() {
                self.handlers[i] = Some(h);
                return Some(i);
            }
        }

        None
    }

    pub fn handle(&'static mut self) {
        for i in 0 .. MAX_SLOT {
            if self.handlers[i].is_some() && self.handlers[i].as_mut().unwrap().invoke() {
                return;
            }
        }
    }
}

// TODO: per-CPU IRQ routing
type SlotNumber = usize;
type VectorNumber = usize;

pub const MAX_VECTOR: VectorNumber = 32;
pub const MAX_SLOT: SlotNumber = 4;

static mut IRQ: [IrqVector; MAX_VECTOR] = [IrqVector::empty(); MAX_VECTOR];

pub fn add(vec: VectorNumber, h: &'static mut dyn IrqHandler) {
    let bind = IrqHandlerBind::new(h);
    unsafe { IRQ[vec].add_slot(bind).unwrap(); }
}

#[no_mangle]
extern "C" fn do_irq_0() {
    use crate::sched;
    sched::r#yield();
}

#[no_mangle]
extern "C" fn do_irq(vec: VectorNumber) {
    unsafe {&mut IRQ[vec as usize]}.handle();
}
