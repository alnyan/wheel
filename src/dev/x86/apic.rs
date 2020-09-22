use core::ptr::{write_volatile, read_volatile, null_mut};
use spin::Mutex;

global_asm!(include_str!("irq_s.S"));

// Must look like a single pointer to ASM world
#[repr(C)]
pub struct LocalApic {
    address: usize
}

pub const REG_ID: usize =           0x20;
pub const REG_TPR: usize =          0x80;
pub const REG_EOI: usize =          0xB0;
pub const REG_SVR: usize =          0xF0;

pub const REG_LVTT: usize =         0x320;

pub const REG_TMRINITCNT: usize =   0x380;
pub const REG_TMRCURRCNT: usize =   0x390;
pub const REG_TMRDIV: usize =       0x3E0;

impl LocalApic {
    // TODO: limits?
    #[inline(always)]
    fn write(&mut self, off: usize, value: u32) {
        unsafe { write_volatile((self.address + off) as *mut u32, value); }
    }

    #[inline(always)]
    fn read(&self, off: usize) -> u32 {
        unsafe { read_volatile((self.address + off) as *mut u32) }
    }

    fn init(&mut self) {
        let tmp = self.read(REG_SVR);
        self.write(REG_SVR, tmp | (1 << 8) | 0xFF);

        // Setup APIC timer
        self.write(REG_TMRDIV, 0x3);
        self.write(REG_LVTT, 32 | (1 << 17));
        self.write(REG_TMRINITCNT, 150000);
        self.write(REG_TMRCURRCNT, 0);
    }
}

#[no_mangle]
static mut apic_eoi: *mut u32 = null_mut();
static APIC: Mutex<LocalApic> = Mutex::new(LocalApic { address: 0 });

pub fn init(address: usize) {
    // TODO: check if already?
    println!("APIC base is 0x{:016x}", address);

    *APIC.lock() = LocalApic {
        address: address
    };
    unsafe { apic_eoi = (address + REG_EOI) as *mut _; }
    APIC.lock().init();
}
