use crate::dev::{irq, io::inb};

struct Keyboard;

impl Keyboard {
    #[inline(always)]
    fn status(&self) -> u8 {
        unsafe {inb(MASTER_STATUS)}
    }

    #[inline(always)]
    fn data(&self) -> u8 {
        unsafe {inb(MASTER_DATA)}
    }
}

impl irq::IrqHandler for Keyboard {
    fn handle(&mut self) -> bool {
        if self.status() & 1 == 0 {
            return false;
        }

        let key = self.data();

        if key < 0x80 {
            println!("press 0x{:02x}", key);
        } else if key == 0xE0 {
            // TODO
        }

        true
    }
}

const MASTER_STATUS:    u16 = 0x64;
const MASTER_DATA:      u16 = 0x60;

static mut MASTER: Keyboard = Keyboard {};

pub fn init() {
    // Bind irq
    irq::add(1, unsafe {&mut MASTER});
}
