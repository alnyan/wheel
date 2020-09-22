use core::marker::PhantomData;
use core::mem::size_of;

use super::base::{Header, Table};

// RSDP + RSDT

#[repr(packed)]
pub struct RootPointer {
    pub signature:      [u8; 8],
    pub checksum:       u8,
    pub oem_id:         [u8; 6],
    pub revision:       u8,
    pub rsdt_address:   u32
}

#[repr(packed)]
pub struct RootSdt<T: Sized> {
    hdr:    Header,
    // Don't know why Rust wants to know size here
    ptrs:   T
}

pub type Rsdt = RootSdt<u32>;

impl Table for Rsdt {
    const SIGNATURE: [u8; 4] = *b"RSDT";
}

pub struct SdtIterator<T: Sized> {
    base:   usize,
    index:  usize,
    limit:  usize,
    _0:     PhantomData<T>
}

impl Iterator for SdtIterator<u32> {
    type Item = &'static mut Header;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.limit {
            return None;
        }

        let ptr = unsafe { *((self.base + self.index * 4) as *mut u32) };
        self.index += 1;
        Some(unsafe { &mut *(ptr as *mut _) })
    }
}

impl RootPointer {
    pub fn is_valid(&self) -> bool {
        self.signature == [b'R', b'S', b'D', b' ',
                           b'P', b'T', b'R', b' ']
        // TODO: checksum
    }
}

impl<T: Sized> RootSdt<T> {
    pub fn iter(&self) -> SdtIterator<T> {
        SdtIterator {
            base:   &self.ptrs as *const _ as usize,
            index:  0,
            limit:  self.hdr.data_size() / size_of::<T>(),
            _0:     PhantomData,
        }
    }
}

