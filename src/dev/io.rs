use core::marker::PhantomData;

pub trait WriteIo<T> {
    fn write(&mut self, t: T);
}

pub trait ReadIo<T> {
    fn read(&self) -> T;
}

pub struct PortIo<T> {
    port: u16,
    _data: PhantomData<T>,
}

impl ReadIo<u8> for PortIo<u8> {
    fn read(&self) -> u8 {
        unsafe { inb(self.port) }
    }
}

impl WriteIo<u8> for PortIo<u8> {
    fn write(&mut self, value: u8) {
        unsafe {
            outb(self.port, value);
        }
    }
}

impl<T> PortIo<T> {
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            _data: PhantomData,
        }
    }
}

/// # Safety
///
/// Absolutely unsafe - arbitrary I/O space reads
#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let mut val: u8;
    llvm_asm!("inb $1, $0":"={al}"(val):"{dx}"(port));
    val
}

/// # Safety
///
/// Absolutely unsafe - arbitrary I/O space writes
#[inline(always)]
pub unsafe fn outb(port: u16, byte: u8) {
    llvm_asm!("outb $0, $1"::"{al}"(byte),"{dx}"(port));
}
