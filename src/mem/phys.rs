use yboot2_proto::MemoryMapInfo;
use core::mem::size_of;
use crate::KERNEL_OFFSET;

const PHYS_MAX_PAGES: usize = 1024 * 1024;

pub (crate) type PhysAddr = usize;

#[derive(PartialEq)]
pub enum PageUsage {
    Reserved,
    Available,
    Kernel,
}

pub struct Page {
    refcount: u32,
    usage: PageUsage
}

#[repr(transparent)]
struct Memory {
    pages: [Page; PHYS_MAX_PAGES]
}

static mut MEMORY: Option<&'static mut Memory> = None;
static mut START_INDEX: usize = 0xFFFFFFFFFFFFFFFF;
static mut END_INDEX: usize = 0;

pub fn alloc_page(usage: PageUsage) -> Option<PhysAddr> {
    assert!(usage != PageUsage::Reserved && usage != PageUsage::Available);

    for index in unsafe { START_INDEX .. END_INDEX } {
        let page = get_page(index);

        if page.usage == PageUsage::Available {
            page.usage = usage;
            page.refcount = 0;
            return Some(index << 12);
        }
    }
    return None;
}

pub fn free_page(phys: PhysAddr) {
    assert!(phys >= unsafe { START_INDEX } && phys <= unsafe { END_INDEX });
}

pub fn get_page(phys: PhysAddr) -> &'static mut Page {
    let num = phys >> 12;
    assert!(num < PHYS_MAX_PAGES);
    return &mut unsafe {MEMORY.as_mut()}.unwrap().pages[num];
}

fn place_struct(at: PhysAddr) {
    unsafe {
        // TODO: better virtualize this pointer
        MEMORY = Some(&mut *(at as *mut Memory));
        for page in &mut MEMORY.as_mut().unwrap().pages {
            page.refcount = 0;
            page.usage = PageUsage::Reserved;
        }
    }
}

extern "C" {
    static _kernel_end: u8;
}

fn kernel_end() -> usize {
    (unsafe { &_kernel_end as *const _ as usize }) - KERNEL_OFFSET
}

fn is_usable(page: PhysAddr) -> bool {
    page > kernel_end()
}

fn fit_mm_pages(mmap: &MemoryMapInfo, req_count: usize) -> Option<PhysAddr> {
    let mut collected = 0usize;
    let mut base_addr = 0usize;

    for item in mmap.iter(true) {
        let aligned_start = (item.begin() + 0xFFF) & !0xFFF;
        let aligned_end = item.end() & !0xFFF;

        if item.is_usable() && aligned_end > aligned_start {
            for page in (aligned_start .. aligned_end).step_by(0x1000) {
                if !is_usable(page) {
                    collected = 0;
                    base_addr = 0;
                    continue;
                }

                if base_addr == 0 {
                    base_addr = page;
                }
                collected += 1;
                if collected == req_count {
                    return Some(base_addr);
                }
            }
        }
    }

    None
}

pub fn init(mmap: &MemoryMapInfo) {
    let pages_addr = fit_mm_pages(mmap, (size_of::<Memory>() + 0xFFF) / 0x1000).unwrap();
    // TODO: make sure fit_mm_pages just doesn't pick addresses which would
    //       screw up the memory map
    assert!(pages_addr > mmap.address as usize + mmap.size as usize ||
            pages_addr + size_of::<Memory>() < mmap.address as usize);

    place_struct(pages_addr);

    let mut total_pages = 0usize;

    for item in mmap.iter(true) {
        let aligned_start = (item.begin() + 0xFFF) & !0xFFF;
        let aligned_end = item.end() & !0xFFF;

        if item.is_usable() && aligned_end > aligned_start {
            for page in (aligned_start .. aligned_end).step_by(0x1000) {
                if !is_usable(page) {
                    continue;
                }

                let index = page >> 12;
                unsafe {
                    if index < START_INDEX {
                        START_INDEX = index;
                    }
                    if index > END_INDEX {
                        END_INDEX = index;
                    }
                }

                let page_struct = get_page(page);
                // Even if mmap pages are crossed now, don't care - mmap is no longer
                // needed
                page_struct.usage = PageUsage::Available;
                page_struct.refcount = 0;

                total_pages += 1;
            }
        }
    }

    println!("Physical memory: {}K available", total_pages * 4);
}
