use core::mem::size_of;
use alloc::boxed::Box;

pub const DEFAULT_KSTACK_PAGES: usize = 2;

#[no_mangle]
static mut CURRENT: *mut InnerContext = core::ptr::null_mut();

// Has to be accessible from assembly using
// well-known offsets
#[repr(C)]
struct InnerContext {
    rsp0:       usize,      // 0x00
    rsp0_top:   usize,      // 0x08
}

pub struct Context {
    inner: InnerContext,
    kstack: Box<[u8]>,
    ustack: Box<[u8]>,
}

impl Context {
    pub fn new(entry: usize) -> Context {
        let mut ctx = Context {
            // Will be initialized a bit later
            inner: InnerContext {
                rsp0: 0,
                rsp0_top: 0,
            },
            kstack: Box::new([0u8; DEFAULT_KSTACK_PAGES * 0x1000]),
            ustack: Box::new([0u8; 2 * 0x1000]),
        };
        ctx.setup(entry);
        ctx
    }

    unsafe fn push(&mut self, val: usize) {
        let base = self.kstack.as_mut_ptr() as usize;
        if self.inner.rsp0 <= base {
            panic!("Context stack overflow");
        }
        self.inner.rsp0 -= size_of::<usize>();
        let ptr = self.inner.rsp0 as *mut usize;
        *ptr = val;
    }

    fn setup(&mut self, entry: usize) {
        // Setup initial rsp0 and rsp0_top
        let base = self.kstack.as_mut_ptr() as usize;
        let ustack_base = self.ustack.as_mut_ptr() as usize;
        let top = base + self.kstack.len();

        self.inner.rsp0 = top;
        self.inner.rsp0_top = top;

        // TODO: argument

        unsafe {
            // Context for iret entry
            // self.push(0x1B);    // ss
            // self.push(ustack_base + 2 * 0x1000);  // user rsp
            // self.push(0x200);     // rflags
            // self.push(0x23);    // cs
            // self.push(entry);   // rip
            self.push(0x10);    // ss
            self.push(ustack_base + 2 * 0x1000);  // user rsp
            self.push(0x200);     // rflags
            self.push(0x08);    // cs
            self.push(entry);   // rip

            // Context for common switching
            self.push(context_entry_iret as usize);

            self.push(0);       // r15
            self.push(0);       // r14
            self.push(0);       // r13
            self.push(0);       // r12
            self.push(0);       // rbp
            self.push(0);       // rbx
        }
    }

    pub unsafe fn switch_to(&mut self, to: &mut Context) {
        context_switch(&mut to.inner, &mut self.inner);
    }

    pub unsafe fn initial_switch(&mut self) {
        context_switch_to(&mut self.inner);
    }
}

extern "C" {
    fn context_switch_to(dst: &mut InnerContext);
    fn context_switch(dst: &mut InnerContext, src: &mut InnerContext);
    fn context_entry_iret();
}

global_asm!(r#"
.type context_entry_iret, %function
.type context_switch, %function
.type context_switch_to, %function
context_entry_iret:
    iretq

.size context_entry_iret, . - context_entry_iret

context_switch:
    // Push callee-saved context
    push %r15
    push %r14
    push %r13
    push %r12
    push %rbp
    push %rbx

    mov %rsp, 0x00(%rsi)
context_switch_to:
    // Load new stack pointer
    mov 0x00(%rdi), %rsp

    // Pop callee-saved context
    pop %rbx
    pop %rbp
    pop %r12
    pop %r13
    pop %r14
    pop %r15

    // TODO: cr3

    // Load new TSS.RSP0 value <- top of kernel stack
    mov 0x08(%rdi), %rax
    mov %rax, (TSS+4)(%rip)

    ret
.size context_switch, . - context_switch
"#);
