use crate::arch::x86::regs;

const MSR_IA32_EFER: u32 = 0xC0000080;
const MSR_IA32_STAR: u32 = 0xC0000081;
const MSR_IA32_LSTAR: u32 = 0xC0000082;
const MSR_IA32_SFMASK: u32 = 0xC0000084;

extern "C" {
    fn syscall_entry();
}

global_asm!(r#"
.section .text
.global syscall_entry
syscall_entry:
    jmp .
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
