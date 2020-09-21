use crate::dev::{PortIo, SerialDevice};
use spin::Mutex;

pub struct SerialPort {
    port: PortIo<u8>
}

impl SerialDevice for SerialPort {
    fn tx(&mut self, byte: u8) {
        use crate::dev::WriteIo;
        self.port.write(byte);
    }
}

pub static COM1: Mutex<SerialPort> = Mutex::new(SerialPort { port: PortIo::new(0x3F8) });
