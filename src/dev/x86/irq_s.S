.macro irq_pushctx
    pushq %r11
    pushq %r10
    pushq %r9
    pushq %r8
    pushq %rcx
    pushq %rdx
    pushq %rsi
    pushq %rdi
    pushq %rax
.endm

.macro irq_popctx
    popq %rax
    popq %rdi
    popq %rsi
    popq %rdx
    popq %rcx
    popq %r8
    popq %r9
    popq %r10
    popq %r11
.endm

.macro irq_entry, n
.type irq_\n, %function
irq_\n:
    cli
    // TODO: swapgs if needed
    irq_pushctx
    mov $\n, %rdi
    call do_irq
    // Do LAPIC EOI on any IRQ,
    // even if unhandled
    mov apic_eoi(%rip), %rax
    movl $0, (%rax)
    irq_popctx
    iretq
.size irq_\n, . - irq_\n
.endm

.section .text
.global irq_0

// IRQ0
.type irq_0, %function
irq_0:
    cli

    irq_pushctx

    // First EOI
    mov apic_eoi(%rip), %rax
    movl $0, (%rax)

    // Then switch
    call do_irq_0

    irq_popctx

    iretq
.size irq_0, . - irq_0

// Regular IRQs
.irp vec, 1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31
    irq_entry \vec
.endr

// Pre-set IRQ vectors
.section .rodata
.global irq_vectors
.type irq_vectors, %object
irq_vectors:
    // Early IRQ0 (before scheduling is available on this CPU)
    .quad irq_0
    // Regular vectors
    .irp vec, 1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31
        .quad irq_\vec
    .endr
.size irq_vectors, . - irq_vectors
