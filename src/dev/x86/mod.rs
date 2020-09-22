pub mod serial;
pub use serial::{COM1, SerialPort};

pub mod apic;
pub use apic::LocalApic;
pub mod ioapic;
pub use ioapic::IoApic;
pub mod acpi;

pub mod ps2;

pub mod irq;
