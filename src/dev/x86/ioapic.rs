use core::ptr::{write_volatile, read_volatile};
use spin::Mutex;

pub struct IoApic {
    address: usize,
    limit: usize        // Maximum GSI number
}

pub struct RedirEntry {
    pub lower: u32,
    pub upper: u32,
}

#[repr(u32)]
pub enum Reg {
    ID          = 0,
    VER         = 1,
    ARB         = 2,
}

impl IoApic {
    #[inline]
    pub fn read(&self, reg: Reg) -> u32 {
        unsafe {
            write_volatile(self.address as *mut u32, reg as u32);
            read_volatile((self.address + 0x10) as *const u32)
        }
    }

    pub fn read_redir(&self, idx: usize, word: usize) -> u32 {
        if word > 1 {
            panic!();
        }

        unsafe {
            write_volatile(self.address as *mut u32, idx as u32 * 2 + 0x10 + word as u32);
            read_volatile((self.address + 0x10) as *const u32)
        }
    }

    pub fn write_redir(&self, idx: usize, word: usize, value: u32) {
        if word > 1 {
            panic!();
        }

        unsafe {
            write_volatile(self.address as *mut u32, idx as u32 * 2 + 0x10 + word as u32);
            write_volatile((self.address + 0x10) as *mut u32, value);
        };
    }

    pub fn set_masked(&mut self, idx: usize, masked: bool) {
        if masked {
            self.write_redir(idx, 0, self.read_redir(idx, 0) | (1 << 16));
        } else {
            self.write_redir(idx, 0, self.read_redir(idx, 0) & !(1 << 16));
        }
    }

    fn init(&mut self) {
        let tmp = self.read(Reg::VER);
        self.limit = ((tmp >> 16) & 0xFFFF) as usize;

        println!("Max GSI number: {}", self.limit);

        // Mask all GSI
        for i in 0 .. self.limit {
            self.set_masked(i, true);
        }

        self.write_redir(1, 0, 33);
    }
}

static IOAPIC: Mutex<IoApic> = Mutex::new(IoApic { address: 0, limit: 0 });

pub fn init(address: usize) {
    println!("I/O APIC base is 0x{:016x}", address);
    *IOAPIC.lock() = IoApic {
        address: address,
        limit: 0
    };
    IOAPIC.lock().init();
}
