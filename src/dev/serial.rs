pub trait SerialDevice {
    fn tx(&mut self, byte: u8);
}
