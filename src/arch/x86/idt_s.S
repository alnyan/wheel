.macro isr_nerr, n
isr_\n:
    cli
    pushq $0
    pushq $\n
    jmp exception_generic
.endm

.macro isr_yerr, n
isr_\n:
    cli
    pushq $\n
    jmp exception_generic
.endm

.section .text
exception_generic:
    // 0x18: cs
    // 0x10: rip
    // 0x08: error code
    // 0x00: error number
    cmpq $0x08, 0x18(%rip)
    jnz 1f
    swapgs
1:

    // Push full CPU context for dump
    pushq %rax
    pushq %rcx
    pushq %rdx
    pushq %rbx
    // RSP already pushed
    pushq %rbp
    pushq %rsi
    pushq %rdi

    pushq %r8
    pushq %r9
    pushq %r10
    pushq %r11
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15

    //movq $AMD64_STACK_CTX_CANARY, %rax
    //pushq %rax

    // amd64_exception(frame)
    movq %rsp, %rdi
    call exception_handler

//    movq $AMD64_STACK_CTX_CANARY, %rax
//    test %rax, (%rsp)
//    jz 1f
//1:
//    addq $8, %rsp

    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %r11
    popq %r10
    popq %r9
    popq %r8

    popq %rdi
    popq %rsi
    popq %rbp

    popq %rbx
    popq %rdx
    popq %rcx
    popq %rax

    addq $16, %rsp

    iretq

isr_nerr 0
isr_nerr 1
isr_nerr 2
isr_nerr 3
isr_nerr 4
isr_nerr 5
isr_nerr 6
isr_nerr 7
isr_yerr 8
isr_nerr 9
isr_yerr 10
isr_yerr 11
isr_yerr 12
isr_yerr 13
isr_yerr 14
isr_nerr 15
isr_nerr 16
isr_yerr 17
isr_nerr 18
isr_nerr 19
isr_nerr 20
isr_nerr 21
isr_nerr 22
isr_nerr 23
isr_nerr 24
isr_nerr 25
isr_nerr 26
isr_nerr 27
isr_nerr 28
isr_nerr 29
isr_yerr 30
isr_nerr 31

.section .rodata
.global exception_vectors
.type exception_vectors, %object
exception_vectors:
    .quad isr_0
    .quad isr_1
    .quad isr_2
    .quad isr_3
    .quad isr_4
    .quad isr_5
    .quad isr_6
    .quad isr_7
    .quad isr_8
    .quad isr_9
    .quad isr_10
    .quad isr_11
    .quad isr_12
    .quad isr_13
    .quad isr_14
    .quad isr_15
    .quad isr_16
    .quad isr_17
    .quad isr_18
    .quad isr_19
    .quad isr_20
    .quad isr_21
    .quad isr_22
    .quad isr_23
    .quad isr_24
    .quad isr_25
    .quad isr_26
    .quad isr_27
    .quad isr_28
    .quad isr_29
    .quad isr_30
    .quad isr_31
.size exception_vectors, . - exception_vectors
