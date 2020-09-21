use crate::arch::x86::{regs, intrinsics};

#[repr(C)]
struct Context {
    r15:        u64,
    r14:        u64,
    r13:        u64,
    r12:        u64,
    r11:        u64,
    r10:        u64,
    r9:         u64,
    r8:         u64,

    rdi:        u64,
    rsi:        u64,
    rbp:        u64,

    rbx:        u64,
    rdx:        u64,
    rcx:        u64,
    rax:        u64,

    exc_no:     u64,
    exc_code:   u64,

    rip:        u64,
    cs:         u64,
    rflags:     u64,
    rsp:        u64,
    ss:         u64
}

fn dump_context(ctx: &Context) {
    print!("\x1b[41m");

    if ctx.exc_no == 14 {
        // Page fault
        let cr2 = regs::cr2::read();
        let cr3 = regs::cr3::read();

        println!("Unhandled page fault");
        println!("Memory space  0x{:016x}", cr3);
        println!("Fault address 0x{:016x}", cr2);
    }

    println!("CPU raised an exception: #{}", ctx.exc_no);

    println!("%rax = 0x{:016x}, %rcx = 0x{:016x}", ctx.rax, ctx.rcx);
    println!("%rdx = 0x{:016x}, %rbx = 0x{:016x}", ctx.rdx, ctx.rbx);

    println!(" %r8 = 0x{:016x},  %r9 = 0x{:016x}",  ctx.r8,  ctx.r9);
    println!("%r10 = 0x{:016x}, %r11 = 0x{:016x}", ctx.r10, ctx.r11);
    println!("%r12 = 0x{:016x}, %r13 = 0x{:016x}", ctx.r12, ctx.r13);
    println!("%r14 = 0x{:016x}, %r15 = 0x{:016x}", ctx.r14, ctx.r15);

    println!("Execution context:");

    println!("%cs:%rip = 0x{:02x}:0x{:016x}", ctx.cs, ctx.rip);
    println!("%ss:%rsp = 0x{:02x}:0x{:016x}", ctx.ss, ctx.rsp);
    println!("%rflags = 0x{:016x}", ctx.rflags);
    print!("\x1b[0m");
}

#[no_mangle]
fn exception_handler(ctx: &mut Context) {
    dump_context(ctx);
    intrinsics::halt();
}
