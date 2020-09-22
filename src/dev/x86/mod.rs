pub mod serial;
pub use serial::{COM1, SerialPort};

pub mod apic;
pub use apic::LocalApic;

pub mod irq;
