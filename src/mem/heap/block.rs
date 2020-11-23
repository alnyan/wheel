use core::convert::TryInto;
use core::mem::size_of;
use core::ptr::null_mut;

pub const MAGIC: u32 = 0xFEDFAD80;
pub const USED: u32 = 1;

pub struct Block {
    pub magic: u32,
    pub size: u32,
    pub prev: *mut Block,
    pub next: *mut Block,
}

impl Block {
    pub fn new(size: usize) -> Block {
        Block {
            magic: MAGIC,
            size: size.try_into().unwrap(),
            prev: null_mut(),
            next: null_mut(),
        }
    }

    pub fn invalid() -> Block {
        Block {
            magic: 0,
            size: 0,
            prev: null_mut(),
            next: null_mut()
        }
    }

    /// # Safety
    ///
    /// Caller must guarantee "at" and "size" define
    /// a valid chunk of memory
    pub unsafe fn place(at: usize, size: usize) -> &'static mut Block {
        let block = &mut *(at as *mut Block);
        *block = Block::new(size);
        block
    }

    pub fn size(&self) -> usize {
        self.size as usize
    }

    pub fn data(&self) -> *mut u8 {
        unsafe { (self as *const Block).offset(1) as *mut u8 }
    }

    pub fn is_used(&self) -> bool {
        self.magic & USED != 0
    }

    pub fn is_valid(&self) -> bool {
        self.magic & MAGIC == MAGIC
    }

    /// Insert `next` after `self` and properly link
    /// blocks.
    pub fn insert(&mut self, next: &mut Block) {
        if !self.next.is_null() {
            let old_next = unsafe { &mut *self.next };
            old_next.prev = next;
        }
        next.next = self.next;
        next.prev = self;
        self.next = next;
    }

    /// Merges `self.next` into `self`, combining
    /// sizes and fixing links (if next is present and
    /// both are unused).
    pub fn merge(&mut self) -> bool {
        if !self.is_used() && !self.next.is_null() {
            let next = unsafe { &mut *self.next };

            if !next.is_used() {
                self.next = next.next;
                if !next.next.is_null() {
                    unsafe { &mut *next.next }.prev = self;
                }
                self.size += size_of::<Block>() as u32 + next.size;

                // Invalidate next block
                *next = Block::invalid();

                return true;
            }
        }
        false
    }

    /// Allocate requested size from self, possibly
    /// splitting it to create a new one
    pub fn alloc(&mut self, count: usize) -> Option<*mut u8> {
        if self.is_used() || self.size() < count {
            return None;
        }

        if self.size() >= count + size_of::<Block>() {
            let new_block_addr = self.data() as usize + count;
            // Should be safe as long as there's no corruption in this header
            let new_block = unsafe {
                Block::place(new_block_addr, self.size() - count - size_of::<Block>())
            };
            // Link
            self.insert(new_block);
            self.size = count as u32;
        }

        self.magic |= USED;
        Some(self.data())
    }
}

#[cfg(test)]
mod test {
    // This module creates Block instances on
    // stack, which is UB, but allowed for testing
    // here
    use crate::block::*;
    use core::mem::size_of;
    use core::ptr::null_mut;

    use std::boxed::*;

    #[test]
    fn empty_block() {
        let b0 = Block::new(0);
        assert!(b0.next == null_mut());
        assert!(b0.prev == null_mut());
        assert!(b0.magic == MAGIC);
        assert!(b0.magic & USED == 0);
        assert!(b0.size == 0);
    }

    #[test]
    fn single_link() {
        let mut b0 = Block::new(0);
        let mut b1 = Block::new(0);

        b0.insert(&mut b1);
        assert!(b0.prev == null_mut());
        assert!(b0.next == &mut b1);
        assert!(b1.prev == &mut b0);
        assert!(b1.next == null_mut());
    }

    #[test]
    fn insert_before_next() {
        let mut b0 = Block::new(0);
        let mut b1 = Block::new(0);
        let mut b2 = Block::new(0);

        b0.insert(&mut b2);
        b0.insert(&mut b1);

        assert!(b0.prev == null_mut());
        assert!(b0.next == &mut b1);
        assert!(b1.prev == &mut b0);
        assert!(b1.next == &mut b2);
        assert!(b2.prev == &mut b1);
        assert!(b2.next == null_mut());
    }

    #[test]
    fn single_merge() {
        let mut b0 = Block::new(32);
        let mut b1 = Block::new(16);

        b0.insert(&mut b1);
        assert!(b0.merge());
        assert!(b0.is_valid());
        assert!(!b1.is_valid());
        assert!(b0.next == null_mut());
        assert!(b0.size == 32 + 16 + size_of::<Block>() as u32);
    }

    #[test]
    fn merge_before_next() {
        let mut b0 = Block::new(24);
        let mut b1 = Block::new(32);
        let mut b2 = Block::new(16);

        b0.insert(&mut b2);
        b0.insert(&mut b1);
        assert!(b0.merge());
        assert!(b0.is_valid());
        assert!(!b1.is_valid());
        assert!(b2.is_valid());
        assert!(b0.next == &mut b2);
        assert!(b2.prev == &mut b0);
        assert!(b0.size == 24 + 32 + size_of::<Block>() as u32);
        assert!(b0.merge());
        assert!(b0.is_valid());
        assert!(!b1.is_valid());
        assert!(!b2.is_valid());
        assert!(b0.next == null_mut());
        assert!(b0.size == 24 + 32 + 16 + 2 * size_of::<Block>() as u32);
    }

    #[test]
    fn no_merge() {
        let mut b0 = Block::new(24);
        assert!(!b0.merge());
    }

    #[test]
    fn data_ptr() {
        let b0 = Block::new(24);
        let data = unsafe { b0.data() };

        let addr0 = &b0 as *const Block as usize;
        let addr1 = data as usize;

        assert!(addr0 + size_of::<Block>() == addr1);
    }

    // The following tests require placement
    #[test]
    fn place() {
        let mut buf = Box::new([0u8; 32768]);
        let ptr0 = buf.as_mut().as_ptr() as usize;
        let block = unsafe { Block::place(ptr0, 32) };
        let ptr1 = block as *mut Block as usize;
        assert!(ptr0 == ptr1);
    }

    #[test]
    fn alloc_no_split() {
        let mut buf = Box::new([0u8; 32768]);
        let buf_ptr = buf.as_mut().as_ptr() as usize;

        let b0 = unsafe { Block::place(buf_ptr, 32) };
        let res = unsafe { b0.alloc(24) };

        assert!(res.is_some());
        assert!(res.unwrap() == unsafe { b0.data() });
        assert!(b0.is_used());
        assert!(b0.next == null_mut());
        assert!(b0.size == 32);
    }

    #[test]
    fn alloc_split() {
        let mut buf = Box::new([0u8; 32768]);
        let buf_ptr = buf.as_mut().as_ptr() as usize;

        let b0 = unsafe { Block::place(buf_ptr, 128) };
        let res = unsafe { b0.alloc(64) };

        assert!(res.is_some());
        assert!(res.unwrap() == unsafe { b0.data() });
        assert!(b0.is_used());
        assert!(b0.next != null_mut());
        assert!(b0.size == 64);

        let b1 = unsafe { &mut *b0.next };

        assert!(b1.is_valid());
        assert!(b1.next == null_mut());
        assert!(b1.prev == b0);
        assert!(b1.size() == 128 - 64 - size_of::<Block>());
    }

    #[test]
    fn no_alloc() {
        let mut buf = Box::new([0u8; 32768]);
        let buf_ptr = buf.as_mut().as_ptr() as usize;

        let b0 = unsafe { Block::place(buf_ptr, 128) };
        assert!(unsafe { b0.alloc(65536) }.is_none());
        assert!(b0.is_valid());
        assert!(!b0.is_used());
        assert!(b0.size == 128);
    }
}
