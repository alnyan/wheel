use crate::arch::x86::regs;

const MSR_IA32_EFER: u32 = 0xC0000080;
const MSR_IA32_STAR: u32 = 0xC0000081;
const MSR_IA32_LSTAR: u32 = 0xC0000082;
const MSR_IA32_SFMASK: u32 = 0xC0000084;

extern "C" {
    fn syscall_entry();
}

#[no_mangle]
extern "C" fn syscall_undefined(no: usize) {
    panic!("Undefined system call: {}", no);
}

global_asm!(r#"
.section .text
.global syscall_entry
syscall_entry:
    // TODO: swapgs n shiet
    // rip -> rcx
    // rflags -> r11

    // Store user stack in temporary location
    mov %rsp, syscall_stack(%rip)
    // Switch to kernel stack
    mov CURRENT(%rip), %rsp // TODO: null check? Don't think this can happen, but still
    mov 0x00(%rsp), %rsp

    // Can do stuff with stack now
    push %rcx
    push %r11
    mov syscall_stack(%rip), %rcx
    push %rcx

    cmp $256, %rax
    jge 1f
    lea SYSCALL_TABLE(%rip), %rcx
    mov (%rcx, %rax, 8), %rcx
    test %rcx, %rcx
    jz 1f

    // Fixup ABI/argument order
    mov %rcx, %rax
    mov %r10, %rcx

    // TODO: interrupts/TSS here?
    call *%rax

    jmp 2f
1:
    mov %rax, %rdi
    call syscall_undefined
2:

    pop %rdi
    pop %r11
    pop %rcx

    mov CURRENT(%rip), %rsp
    mov %rdi, 0x00(%rsp)
    mov %rdi, %rsp

    // TODO: swapgs back

    sysretq

.section .data
syscall_stack:
    .quad 0
"#);

pub fn init() {
    unsafe {
        // Syscall entry
        regs::wrmsr(MSR_IA32_LSTAR, syscall_entry as usize as u64);
        // Mask out IF each time syscall is executed
        regs::wrmsr(MSR_IA32_SFMASK, 1 << 9);
        // Segment registers to use when switching to syscall context
        regs::wrmsr(MSR_IA32_STAR, ((0x1Bu64 - 8) << 48) | (0x08u64 << 32));

        // Enable syscalls by setting SCE bit in EFER
        regs::wrmsr(MSR_IA32_EFER, regs::rdmsr(MSR_IA32_EFER) | 1);
    }
}
