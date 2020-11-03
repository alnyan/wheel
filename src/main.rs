#![feature(
    llvm_asm,
    global_asm,
    const_in_array_repeat_expressions,
    const_fn,
    alloc_error_handler,
)]
#![no_main]
#![no_std]

extern crate alloc;
extern crate yboot2_proto;

pub const KERNEL_OFFSET: usize = 0xFFFFFF0000000000;
static mut FB: usize = 0;

#[inline(always)]
pub fn virtualize(phys: usize) -> usize {
    assert!(phys < 0x100000000); // Because above 4GiB isn't mapped yet
    phys + KERNEL_OFFSET
}

#[macro_use]
pub mod debug;
pub mod arch;
mod boot;
pub mod dev;
pub mod mem;
pub mod sync;
pub mod thread;
pub mod syscall;

fn task1(_: usize) {
    loop {
        unsafe { llvm_asm!("syscall"); }
        //println!("1");
    }
}

fn task2(_: usize) {
    let boot = boot::boot_data();
    let mut p = 0;
    let mut t = 0;
    let off = (boot.video.width * boot.video.height) as usize * 2;
    loop {
        let ptr = unsafe { FB } + off + p * 4;
        p += 1;
        if p >= (boot.video.width * boot.video.height) as usize / 2 {
            p = 0;
        }
        unsafe { *(ptr as *mut u32) = t & 0xFFFFFF; }
        for i in 0 .. 100 {
            unsafe { llvm_asm!("nop"); }
        }
        t += 1;
        //println!("2");
    }
}

#[no_mangle]
pub extern "C" fn kernel_main() {
    // Cleanup terminal colors after UEFI
    print!("\x1b[0m\x1b[2J\x1B[0;0f");
    let boot = boot::boot_data();
    use yboot2_proto::Magic;
    assert!(boot.hdr.loader_magic == yboot2_proto::ProtoV1::LOADER_MAGIC);
    assert!(boot.video.framebuffer != 0);
    unsafe { FB = virtualize(boot.video.framebuffer as usize); }

    arch::x86::gdt::init();
    arch::x86::idt::init();

    mem::phys::init(&boot.memory_map);
    mem::heap::init_somewhere(1024 * 1024 * 4);

    // Initialize local APIC
    dev::x86::apic::init(virtualize(0xFEE00000));
    dev::x86::acpi::init(Some(boot.rsdp as usize));
    dev::x86::ps2::init();

    syscall::init();

    let mut thr0 = thread::Thread::new(task1, 0);
    let mut thr1 = thread::Thread::new(task2, 0);
    // Enter the thread
    unsafe {
        thread::enqueue(&mut thr0);
        thread::enqueue(&mut thr1);
        thread::enter();
    }

    panic!("Did not enter the thread");
}

#[panic_handler]
fn panic_handler(pi: &core::panic::PanicInfo) -> ! {
    println!("{}", pi);
    loop {}
}
