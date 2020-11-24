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
pub mod proc;
pub mod sched;
pub mod syscall;

fn task1(_: usize) {
    loop {
    }
}

fn task2(_: usize) {
    loop {
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
    sched::init();

    // Will not get freed until the end of scope, so okay
    let mut p0 = proc::Process::kspawn(task1, 0, "task1");
    let mut p1 = proc::Process::kspawn(task2, 0, "task1");

    sched::queue(&mut p0);
    sched::queue(&mut p1);
    //let mut proc = Process::new_kernel();
    //proc.spawn(task1 as usize, 0).unwrap();
    //proc.spawn(task2 as usize, 0).unwrap();

    // Enter the thread
    unsafe {
        sched::enter();
    }
}

#[panic_handler]
fn panic_handler(pi: &core::panic::PanicInfo) -> ! {
    println!("{}", pi);
    loop {}
}
