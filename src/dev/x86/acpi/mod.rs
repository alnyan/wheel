use crate::virtualize;

pub mod base;
pub use base::*;
pub mod tables;
pub use tables::*;
pub mod ptr;
pub use ptr::*;

use spin::Mutex;

pub static FADT: Mutex<Option<&'static mut Fadt>> = Mutex::new(None);
pub static MADT: Mutex<Option<&'static mut Madt>> = Mutex::new(None);

fn init_rsdp(addr: usize) {
    let rsdp = unsafe { &*(addr as *const RootPointer) };
    if !rsdp.is_valid() {
        panic!("ACPI root pointer is invalid");
    }

    let rsdt = unsafe { &*(virtualize(rsdp.rsdt_address as usize) as *const Rsdt) };
    assert!(rsdt.is_valid());

    for item in rsdt.iter() {
        // TODO: rewrite to match?
        if let Some(table) = item.table::<Fadt>() {
            *FADT.lock() = Some(table);
        }
        if let Some(table) = item.table::<Madt>() {
            *MADT.lock() = Some(table);
        }
    }
}

pub fn init(from_loader: Option<usize>) {
    // TODO: RSDP location if not provided
    let rsdp_address = from_loader.unwrap();
    init_rsdp(rsdp_address);

    // Iterate MADT to find I/O APIC record
    if let Some(madt) = &*MADT.lock() {
        use crate::dev::x86::ioapic;

        for rec in madt.iter() {
            if let MadtRecord::IoApic(id, addr, base) = rec {
                ioapic::init(virtualize(addr as usize));
            }
        }
    }
}
