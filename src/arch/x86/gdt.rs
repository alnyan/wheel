#![allow(dead_code)]

use core::mem::size_of;

#[repr(packed)]
struct Entry64 {
    limit_lo:   u16,
    base_lo:    u16,
    base_mi:    u8,
    access:     u8,
    flags:      u8,
    base_hi:    u8
}

#[repr(packed)]
struct Tss {
    _res0:      u32,
    rsp0:       u64,
    rsp1:       u64,
    rsp2:       u64,
    _res1:      u64,
    ist1:       u64,
    ist2:       u64,
    ist3:       u64,
    ist4:       u64,
    ist5:       u64,
    ist6:       u64,
    ist7:       u64,
    _res2:      u64,
    _res3:      u16,
    iopb_base:  u16
}

#[repr(packed)]
struct Pointer {
    size:   u16,
    offset: usize
}

// TODO: cfg
type Entry = Entry64;

impl Entry64 {
    pub const fn new(base: u32, limit: u32, flags: u8, access: u8) -> Self {
        Self {
            base_lo:    (base & 0xFFFF) as u16,
            base_mi:    ((base >> 16) & 0xFF) as u8,
            base_hi:    ((base >> 24) & 0xFF) as u8,
            access:     access,
            flags:      (flags & 0xF0) | (((limit >> 16) & 0xF) as u8),
            limit_lo:   (limit & 0xFFFF) as u16,
        }
    }
}

const ACC_AC: u8 = 1 << 0;
const ACC_RW: u8 = 1 << 1;
const ACC_EX: u8 = 1 << 3;
const ACC_S:  u8 = 1 << 4;
const ACC_R3: u8 = 3 << 5;
const ACC_PR: u8 = 1 << 7;

const FLAG_LONG: u8 = 1 << 5;

const ENTRY_COUNT: usize = 7;
static mut ENTRIES: [Entry; ENTRY_COUNT] = [
    Entry::new(0, 0, 0, 0),                             // global null 0x00
    Entry::new(0, 0, FLAG_LONG,                         // kernel code 0x08
                     ACC_PR | ACC_S | ACC_EX),
    Entry::new(0, 0, 0,                                 // kernel data 0x10
                     ACC_PR | ACC_S | ACC_RW),
    Entry::new(0, 0, 0,                                 // user   code 0x18/0x1B
                     ACC_PR | ACC_S | ACC_RW | ACC_R3),
    Entry::new(0, 0, FLAG_LONG,                         // use    data 0x20/0x23
                     ACC_PR | ACC_S | ACC_EX | ACC_R3),
    Entry::new(0, 0, 0, 0),                             // Empty TSS
    Entry::new(0, 0, 0, 0),                             // Empty TSS
];
static TSS: Tss = Tss {
    _res0:      0,
    rsp0:       0,
    rsp1:       0,
    rsp2:       0,
    _res1:      0,
    ist1:       0,
    ist2:       0,
    ist3:       0,
    ist4:       0,
    ist5:       0,
    ist6:       0,
    ist7:       0,
    _res2:      0,
    _res3:      0,
    iopb_base:  0
};
static mut POINTER: Pointer = Pointer {
    offset: 0,
    size: 0
};

extern "C" {
    #[allow(improper_ctypes)]
    fn load_gdt(ptr: *const Pointer);
}
global_asm!(include_str!("gdt_s.S"));

pub fn init() {
    // TODO: TSS
    unsafe {
        let tss_ptr = &TSS as *const _ as usize;
        ENTRIES[ENTRY_COUNT - 2] = Entry::new((tss_ptr & 0xFFFFFFFF) as u32,
                                              (size_of::<Tss>() - 1) as u32,
                                              FLAG_LONG,
                                              ACC_PR | ACC_AC | ACC_EX);
        let tss_upper = &mut ENTRIES[ENTRY_COUNT - 1] as *mut _ as *mut u64;
        *tss_upper = ((tss_ptr >> 32) & 0xFFFFFFFF) as u64;

        POINTER.offset = ENTRIES.as_ptr() as usize;
        POINTER.size = (ENTRY_COUNT * size_of::<Entry>() - 1) as u16;
        load_gdt(&POINTER);
    }
}
