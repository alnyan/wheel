use crate::dev::{x86::COM1, SerialDevice};
use core::fmt;

struct SerialWriter<'a, T: SerialDevice> {
    port: &'a mut T
}

impl<'a, T: SerialDevice> fmt::Write for SerialWriter<'a, T> {
    fn write_str(&mut self, data: &str) -> fmt::Result {
        for byte in data.bytes() {
            self.port.tx(byte);
        }
        Ok(())
    }
}

pub fn write_fmt(args: fmt::Arguments) -> fmt::Result {
    use core::fmt::Write;
    let mut wr = SerialWriter {
        port: &mut *COM1.lock()
    };
    wr.write_fmt(args)
}

#[macro_export]
macro_rules! print {
    ($($args:tt)*) => ($crate::debug::write_fmt(format_args!($($args)*)).unwrap())
}

#[macro_export]
macro_rules! println {
    ($($args:tt)*) => (print!("{}\n", format_args!($($args)*)))
}
