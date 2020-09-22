use core::mem::size_of;

#[repr(packed)] // C?
pub struct Header {
    signature:      [u8; 4],
    length:         u32,
    revision:       u8,
    checksum:       u8,
    oem_id:         [u8; 6],
    oem_table_id:   [u8; 8],
    oem_revision:   u32,
    creator_id:     u32,
    creator_rev:    u32
}

pub trait Table {
    const SIGNATURE: [u8; 4];

    fn is_valid(&self) -> bool {
        true
    }
}

impl Header {
    pub fn data_size(&self) -> usize {
        assert!(self.length as usize >= size_of::<Header>());
        self.length as usize - size_of::<Header>()
    }

    pub fn table<T: Table>(&mut self) -> Option<&'static mut T> {
        if T::SIGNATURE == self.signature {
            let self_addr = unsafe {self as *mut _ as usize};
            Some(unsafe { &mut *(self_addr as *mut _) })
        } else {
            None
        }
    }
}

