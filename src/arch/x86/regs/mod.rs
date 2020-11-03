pub mod cr2;
pub mod cr3;

pub unsafe fn rdmsr(r: u32) -> u64 {
    let mut res: u64;
    llvm_asm!("rdmsr":"=A"(res):"{rcx}"(r):"rdx");
    res
}

pub unsafe fn wrmsr(r: u32, v: u64) {
    let low = (v & 0xFFFFFFFF) as u32;
    let high = (v >> 32) as u32;
    llvm_asm!("wrmsr"::"{rax}"(low),"{rdx}"(high),"{rcx}"(r):"rdx");
}
