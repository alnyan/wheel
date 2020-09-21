use core::marker::PhantomData;

pub trait WriteIo<T> {
    fn write(&mut self, t: T);
}

pub trait ReadIo<T> {
    fn read(&self) -> T;
}

pub struct PortIo<T> {
    port: u16,
    _data: PhantomData<T>
}

//pub struct MemoryIo<T> {
//    address: usize,
//    _data: PhantomData<T>
//}

impl ReadIo<u8> for PortIo<u8> {
    fn read(&self) -> u8 {
        let mut val: u8;
        unsafe { llvm_asm!("inb $0, $1":"={al}"(val):"{dx}"(self.port)); }
        val
    }
}

impl WriteIo<u8> for PortIo<u8> {
    fn write(&mut self, value: u8) {
        unsafe { llvm_asm!("outb $0, $1"::"{al}"(value),"{dx}"(self.port)); }
    }
}

impl<T> PortIo<T> {
    pub const fn new(port: u16) -> Self {
        Self {
            port: port,
            _data: PhantomData
        }
    }
}
