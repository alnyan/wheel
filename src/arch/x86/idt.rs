#![allow(dead_code)]

use core::mem::size_of;

#[repr(packed)]
struct Entry {
    base_lo:    u16,
    selector:   u16,
    zero0:      u8,
    flags:      u8,
    base_hi:    u16,
    base_ex:    u32,
    zero1:      u32
}

#[repr(packed)]
struct Pointer {
    limit: u16,
    offset: usize
}

impl Entry {
    pub const fn new(addr: usize, selector: u16, flags: u8) -> Self {
        Self {
            base_lo:    (addr & 0xFFFF) as u16,
            base_hi:    ((addr >> 16) & 0xFFFF) as u16,
            base_ex:    ((addr >> 32) & 0xFFFFFFFF) as u32,
            zero0:      0,
            zero1:      1,
            selector:   selector,
            flags:      flags
        }
    }
    pub const fn empty() -> Self {
        Self {
            base_lo:    0,
            selector:   0,
            zero0:      0,
            flags:      0,
            base_hi:    0,
            base_ex:    0,
            zero1:      0
        }
    }
}

const FLAG_TASK32: u8   = 5;
const FLAG_INT32: u8    = 14;
const FLAG_PR: u8       = 1 << 7;

const ENTRY_COUNT: usize = 256;
static mut ENTRIES: [Entry; ENTRY_COUNT] = [Entry::empty(); ENTRY_COUNT];
static mut POINTER: Pointer = Pointer {
    limit:  0,
    offset: 0,
};

extern "C" {
    static exception_vectors: [usize; 32];
}
global_asm!(include_str!("idt_s.S"));

pub fn init() {
    for i in 0 .. 32 {
        unsafe {
            ENTRIES[i] = Entry::new(exception_vectors[i] as usize,
                                    0x08, FLAG_PR | FLAG_INT32);
        }
    }

    unsafe {
        POINTER.offset = ENTRIES.as_ptr() as usize;
        POINTER.limit = (ENTRY_COUNT * size_of::<Entry>() - 1) as u16;

        llvm_asm!("lidt ($0)"::"{rdi}"(&POINTER):"memory");
    }
}
