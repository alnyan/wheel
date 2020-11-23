use crate::mem::phys::{self, PageUsage};
use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;
use crate::virtualize;

pub mod zone;
pub mod block;

pub use zone::Zone;
pub use block::Block;

// 16MiB
const HEAP_ZONES: usize = 4;

struct KernelHeap {
    zones: [RefCell<Zone>; HEAP_ZONES],
}

impl KernelHeap {}

unsafe impl GlobalAlloc for KernelHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        // TODO: align
        let _align = layout.align();

        println!("alloc({})", size);
        for cell in self.zones.iter() {
            if let Some(ptr) = cell.borrow_mut().alloc(size) {
                return ptr;
            }
        }
        core::ptr::null_mut()
    }
    // TODO: free()
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static mut HEAP: KernelHeap = KernelHeap {
    zones: [RefCell::new(Zone::empty()); HEAP_ZONES],
};

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}

/// # Safety
///
/// Caller must guarantee "at" and "size" define
/// a valid chunk of memory
pub unsafe fn init(at: usize, size: usize) {
    let zone_size = size / HEAP_ZONES;
    if zone_size % 4096 != 0 {
        panic!("Zone size is not page-aligned");
    }

    for (i, cell) in HEAP.zones.iter().enumerate() {
        *cell.borrow_mut() = Zone::place(at + zone_size * i, zone_size);
    }
}

pub fn init_somewhere(zone_size: usize) {
    if zone_size % 4096 != 0 {
        panic!("Zone size is not page-aligned");
    }

    println!("Test");
    for cell in unsafe { &HEAP }.zones.iter() {
        if let Some(phys_base) = phys::alloc_contiguous(PageUsage::Kernel, zone_size / 4096) {
            *cell.borrow_mut() = unsafe { Zone::place(virtualize(phys_base), zone_size) };
        } else {
            panic!("Failed to allocate {} contiguous physical pages for heap zone", zone_size / 4096);
        }
    }
}
