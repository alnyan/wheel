//! Virtual memory table management stuff

use core::marker::PhantomData;
use crate::virtualize;

pub mod phys;

/// Page table entry is valid
pub const PAGE_PRESENT: u64     = 1 << 0;
/// Depending on translation level may mean
/// alternate page size (only 2MiB pages
/// are used so far)
pub const PAGE_HUGE: u64        = 1 << 7;

/// A single specific page table nesting level
pub trait Level {
    const INDEX_SHIFT: usize;
}
/// Address translation logic
pub trait Table {
    fn translate(&self, virt: usize, flags: Option<&mut u64>) -> Option<usize>;
}

/// A wrapper for [u64; 512] page table providing
/// translation semantics
#[repr(transparent)]
pub struct PageTable<T: Level> {
    entries:    [u64; 512],
    _0:         PhantomData<T>
}

/// Page table
pub struct L1;
/// Page directory
pub struct L2;
/// Page directory pointer table
pub struct L3;
/// PML4
pub struct L4;

/// Root level of page tables, covers the whole memory space
type Space = PageTable<L4>;

impl Level for L1 {
    const INDEX_SHIFT: usize = 12;
}
impl Level for L2 {
    const INDEX_SHIFT: usize = 21;
}
impl Level for L3 {
    const INDEX_SHIFT: usize = 30;
}
impl Level for L4 {
    const INDEX_SHIFT: usize = 39;
}

impl<T: Level> Table for PageTable<T> {
    fn translate(&self, virt: usize, flags: Option<&mut u64>) -> Option<usize> {
        let index = (virt >> T::INDEX_SHIFT) & 0x1FF;
        let entry = self.entries[index];

        if entry & PAGE_PRESENT != 0 {
            if flags.is_some() {
                // TODO: NX
                *flags.unwrap() = entry & 0xFFF;
            }

            Some((entry & !0xFFF) as usize)
        } else {
            None
        }
    }
}

pub static mut KERNEL: Option<&'static mut Space> = None;

// TODO: add a way to ignore 2MiB pages without using flags
/// Perform full address translation up to first physical page
pub fn translate(space: &Space, virt: usize, flags: Option<&mut u64>) -> Option<usize> {
    let mut inner_flags = 0u64;
    if let Some(l4_addr) = space.translate(virt, Some(&mut inner_flags)) {
        if inner_flags & PAGE_HUGE != 0 {
            panic!();   // Not allowed at this level
        }

        let pdpt = unsafe { &*(virtualize(l4_addr) as *const PageTable<L3>) };
        if let Some(l3_addr) = pdpt.translate(virt, Some(&mut inner_flags)) {
            if inner_flags & PAGE_HUGE != 0 {
                panic!();       // Sometimes allowed, but we don't do this here
            }

            let pd = unsafe { &*(virtualize(l3_addr) as *const PageTable<L2>) };
            if let Some(l2_addr) = pd.translate(virt, Some(&mut inner_flags)) {
                if inner_flags & PAGE_HUGE != 0 {
                    // 2MiB page
                    flags.map(|r| {*r = inner_flags});
                    Some(l2_addr | (virt & 0x1FFFFF))
                } else {
                    let pt = unsafe { &*(virtualize(l2_addr) as *const PageTable<L1>) };
                    return pt.translate(virt, flags);
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}
