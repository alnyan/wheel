use core::ptr::{write_volatile, read_volatile, null_mut};
use spin::Mutex;

pub struct LocalApic {
    address: usize
}

#[repr(usize)]
pub enum Reg {
    ID = 0x20,
    TPR = 0x80,
    EOI = 0xB0,
    SVR = 0xF0,

    LVTT = 0x320,

    TMRINITCNT = 0x380,
    TMRCURRCNT = 0x390,
    TMRDIV = 0x3E0
}

impl LocalApic {
    #[inline(always)]
    fn write(&mut self, reg: Reg, value: u32) {
        unsafe { write_volatile((self.address + reg as usize) as *mut u32, value); }
    }

    #[inline(always)]
    fn read(&self, reg: Reg) -> u32 {
        unsafe { read_volatile((self.address + reg as usize) as *mut u32) }
    }

    fn init(&mut self) {
        let tmp = self.read(Reg::SVR);
        self.write(Reg::SVR, tmp | (1 << 8) | 0xFF);

        // Setup APIC timer
        self.write(Reg::TMRDIV, 0x3);
        self.write(Reg::LVTT, 32 | (1 << 17));
        self.write(Reg::TMRINITCNT, 150000);
        self.write(Reg::TMRCURRCNT, 0);
    }
}

#[no_mangle]
static mut apic_eoi: *mut u32 = null_mut();
static APIC: Mutex<LocalApic> = Mutex::new(LocalApic { address: 0 });

pub fn init(address: usize) {
    // TODO: check if already?
    println!("APIC base is 0x{:016x}", address);

    *APIC.lock() = LocalApic { address };
    unsafe { apic_eoi = (address + Reg::EOI as usize) as *mut _; }
    APIC.lock().init();
}
