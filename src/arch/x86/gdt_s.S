.section .text
.global load_gdt
.type load_gdt, %function

load_gdt:
    lgdt (%rdi)
    movw $0x28, %ax
    ltr %ax

    mov %rsp, %rdx
    mov $0x10, %rax
    push %rax
    push %rdx

    pushf

    mov $0x08, %rax
    push %rax
    lea 1f(%rip), %rax
    push %rax

    iretq

1:
    mov $0x10, %rax
    mov %rax, %ds
    mov %rax, %es
    mov %rax, %fs
    mov %rax, %gs
    ret

.size load_gdt, . - load_gdt
