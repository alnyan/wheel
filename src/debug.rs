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
    ()              => (print!("\n"));
    ($($args:tt)*)  => (print!("{}\n", format_args!($($args)*)))
}

// TODO: offsets
unsafe fn dump_line(base: *const u8, count: usize) {
    for i in 0 .. 16 {
        if i < count {
            print!("{:02x}", *(base.offset(i as isize)));
        } else {
            print!("  ");
        }
        if i % 2 != 0 {
            print!(" ");
        }
    }
    println!();
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
