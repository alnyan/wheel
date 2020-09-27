use super::base::{Table, Header};
use core::mem::size_of;

#[repr(packed)]
pub struct Fadt {
    pub hdr:            Header,
    pub firmware_ctrl:  u32,
    pub dsdt:           u32,
    _res0:              u8,
    // ...
}

impl Table for Fadt {
    const SIGNATURE: [u8; 4] = *b"FACP";
}

////

#[repr(packed)]
pub struct Madt {
    pub hdr:            Header,
    pub local_apic:     u32,
    pub flags:          u32,
    records:            u8
}

pub struct MadtIterator {
    base:       usize,
    offset:     usize,
    limit:      usize
}

pub enum MadtRecord {
    LocalApic(u8, u8, u32),
    IoApic(u8, u32, u32),
    Unknown(u8, u8)
}

impl Table for Madt {
    const SIGNATURE: [u8; 4] = *b"APIC";
}

impl Madt {
    pub fn iter(&self) -> MadtIterator {
        MadtIterator {
            base: &self.records as *const _ as usize,
            offset: 0,
            limit: self.hdr.data_size() - size_of::<u32>() * 2
        }
    }
}

impl Iterator for MadtIterator {
    type Item = MadtRecord;

    fn next(&mut self) -> Option<MadtRecord> {
        if self.offset >= self.limit {
            return None;
        }

        let ptr = (self.base + self.offset) as *const u8;
        let (kind, len) = unsafe {
            (*ptr, *ptr.offset(1))
        };

        let res = match kind {
            1   => Some(unsafe {
                MadtRecord::IoApic(*ptr.offset(2),
                                   *(ptr.offset(4) as *const _),
                                   *(ptr.offset(8) as *const _))
            }),
            _   => Some(MadtRecord::Unknown(kind, len))
        };

        self.offset += len as usize;

        return res;
    }
}
