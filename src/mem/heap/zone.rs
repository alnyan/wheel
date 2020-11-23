use super::Block;
use core::marker::PhantomData;
use core::ptr::null_mut;
use core::mem::size_of;
use spin::Mutex;

pub struct Zone {
    head: Mutex<*mut Block>
}
pub struct Stat {
    blocks: usize,
    alloc: usize,
    bytes_alloc: usize,
    bytes_free: usize
}

struct BlockIterMut<'a> {
    current: *mut Block,
    _phantom: PhantomData<&'a Block>
}
struct BlockIter<'a> {
    current: *const Block,
    _phantom: PhantomData<&'a Block>
}

impl Zone {
    pub fn new(head: *mut Block) -> Zone {
        Zone {
            head: Mutex::new(head)
        }
    }

    pub const fn empty() -> Zone {
        Zone {
            head: Mutex::new(null_mut())
        }
    }

    /// # Safety
    ///
    /// Caller must guarantee "at" and "size" define
    /// a valid chunk of memory
    pub unsafe fn place(at: usize, size: usize) -> Zone {
        let head = Block::place(at, size - size_of::<Block>());
        Zone {
            head: Mutex::new(head)
        }
    }

    fn iter_mut(&mut self) -> BlockIterMut {
        BlockIterMut {
            current: *self.head.lock(),
            _phantom: PhantomData
        }
    }
    fn iter(&self) -> BlockIter {
        BlockIter {
            current: *self.head.lock(),
            _phantom: PhantomData
        }
    }

    pub fn stat(&self, st: &mut Stat) {
        st.blocks = 0;
        st.alloc = 0;
        st.bytes_alloc = 0;
        st.bytes_free = 0;

        for block in self.iter() {
            st.blocks += 1;
            if block.is_used() {
                st.alloc += 1;
                st.bytes_alloc += block.size();
            } else {
                st.bytes_free += block.size();
            }
        }
    }

    pub fn alloc(&mut self, size: usize) -> Option<*mut u8> {
        for block in self.iter_mut() {
            if let Some(data) = block.alloc(size) {
                return Some(data);
            }
        }
        None
    }
}

impl Stat {
    pub const fn new() -> Stat {
        Stat {
            blocks: 0,
            alloc: 0,
            bytes_alloc: 0,
            bytes_free: 0
        }
    }
}

impl<'a> Iterator for BlockIterMut<'a> {
    type Item = &'a mut Block;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            None
        } else {
            let r = unsafe { &mut *self.current };
            self.current = r.next;
            Some(r)
        }
    }
}
impl<'a> Iterator for BlockIter<'a> {
    type Item = &'a Block;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            None
        } else {
            let r = unsafe { &*self.current };
            self.current = r.next;
            Some(r)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::zone::{Zone, Stat};
    use crate::block::Block;
    use core::ptr::null_mut;
    use core::mem::size_of;
    use std::boxed::Box;

    #[test]
    fn place() {
        let mut buf = Box::new([0u8; 32768]);
        let buf_ptr = buf.as_mut().as_ptr() as usize;

        let zone = unsafe { Zone::place(buf_ptr, 32768) };

        assert!(*zone.head.lock() as usize == buf_ptr);
        let block = unsafe { &mut **zone.head.lock() };
        assert!(block.size() == 32768 - size_of::<Block>());
        assert!(block.is_valid());
    }

    #[test]
    fn alloc() {
        let mut buf = Box::new([0u8; 32768]);
        let buf_ptr = buf.as_mut().as_ptr() as usize;

        let mut zone = unsafe { Zone::place(buf_ptr, 32768) };
        let res = unsafe { zone.alloc(256) };
        let block = unsafe { &mut **zone.head.lock() };
        assert!(res.is_some());
        assert!(res.unwrap() == unsafe { block.data() });
        assert!(block.is_used());
        assert!(block.size() == 256);
    }

    #[test]
    fn empty_stat() {
        let mut buf = Box::new([0u8; 32768]);
        let buf_ptr = buf.as_mut().as_ptr() as usize;

        let mut zone = unsafe { Zone::place(buf_ptr, 32768) };
        let mut st0 = Stat::new();

        zone.stat(&mut st0);
        assert!(st0.blocks == 1);
        assert!(st0.alloc == 0);
        assert!(st0.bytes_free == 32768 - size_of::<Block>());
        assert!(st0.bytes_alloc == 0);
    }

    #[test]
    fn stat() {
        let mut buf = Box::new([0u8; 32768]);
        let buf_ptr = buf.as_mut().as_ptr() as usize;

        let mut zone = unsafe { Zone::place(buf_ptr, 32768) };
        unsafe {
            zone.alloc(16);
            zone.alloc(24);
            zone.alloc(32);
        }
        let mut st0 = Stat::new();

        zone.stat(&mut st0);
        assert!(st0.blocks == 4);
        assert!(st0.alloc == 3);
        assert!(st0.bytes_free == 32768 - 4 * size_of::<Block>() - 16 - 24 - 32);
        assert!(st0.bytes_alloc == 16 + 24 + 32);
    }
}
