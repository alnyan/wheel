use crate::dev::{x86::COM1, SerialDevice};
use crate::sync::IrqDisable;
use core::fmt;

pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
    Fatal
}

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

fn write_fmt_raw(args: fmt::Arguments) -> fmt::Result {
    use core::fmt::Write;
    let _lock = IrqDisable::new();
    let mut wr = SerialWriter {
        port: &mut *COM1.lock()
    };
    wr.write_fmt(args)
}

pub fn write_fmt(level: Level, file: &str, line: u32, args: fmt::Arguments) -> fmt::Result {
    use core::fmt::Write;
    let _lock = IrqDisable::new();
    let mut wr = SerialWriter {
        port: &mut *COM1.lock()
    };
    match level {
        Level::Warn     => wr.write_str("\x1b[33;1m")?,
        Level::Error    => wr.write_str("\x1b[31;1m")?,
        Level::Fatal    => wr.write_str("\x1b[41;1m")?,
        _               => (),
    }
    wr.write_fmt(format_args!("[{}:{}] ", file, line))?;
    wr.write_fmt(args)?;
    // TODO: only when necessary
    wr.write_str("\x1b[0m")
}

#[macro_export]
macro_rules! print {
    ($($args:tt)*) => ($crate::debug::write_fmt($crate::debug::Level::Debug,
                                                file!(),
                                                line!(),
                                                format_args!($($args)*)).unwrap())
}

#[macro_export]
macro_rules! info {
    ($($args:tt)*) => ($crate::debug::write_fmt($crate::debug::Level::Info,
                                                file!(),
                                                line!(),
                                                format_args!($($args)*)).unwrap())
}

#[macro_export]
macro_rules! warn {
    ($($args:tt)*) => ($crate::debug::write_fmt($crate::debug::Level::Warn,
                                                file!(),
                                                line!(),
                                                format_args!($($args)*)).unwrap())
}

#[macro_export]
macro_rules! error {
    ($($args:tt)*) => ($crate::debug::write_fmt($crate::debug::Level::Error,
                                                file!(),
                                                line!(),
                                                format_args!($($args)*)).unwrap())
}

#[macro_export]
macro_rules! fatal {
    ($($args:tt)*) => ($crate::debug::write_fmt($crate::debug::Level::Fatal,
                                                file!(),
                                                line!(),
                                                format_args!($($args)*)).unwrap())
}

#[macro_export]
macro_rules! println {
    ()              => (print!("\n"));
    ($($args:tt)*)  => (print!("{}\n", format_args!($($args)*)))
}

// TODO: offsets
unsafe fn dump_line(base: *const u8, count: usize) {
    for i in 0 .. 16 {
        if i < count {
            write_fmt_raw(format_args!("{:02x}", *(base.offset(i as isize))));
        } else {
            write_fmt_raw(format_args!("  "));
        }
        if i % 2 != 0 {
            write_fmt_raw(format_args!(" "));
        }
    }
    write_fmt_raw(format_args!("\n"));
}

pub unsafe fn dump(ptr: usize, count: usize) {
    let full_lines = count / 16;
    for i in 0 .. full_lines {
        dump_line((ptr + i * 16) as *const _, 16);
    }
    if (count % 16) != 0 {
        dump_line((ptr + full_lines * 16) as *const _, count % 16);
    }
}
