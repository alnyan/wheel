ENTRY(kernel_main);

KERNEL_OFFSET = 0xFFFFFF0000000000;
KERNEL_PHYSBASE = 0x200000;     /* 2MiB */

SECTIONS {
    . = KERNEL_OFFSET + KERNEL_PHYSBASE;

    .text : AT(ADDR(.text) - KERNEL_OFFSET) ALIGN(4K) {
        *(.text*)
        *(.rodata*)
    }

    .data : AT(ADDR(.data) - KERNEL_OFFSET) ALIGN(4K) {
        *(.data*)
    }

    .boot : AT(ADDR(.boot) - KERNEL_OFFSET) {
        KEEP(*(.boot))
    }

    .bss : AT(ADDR(.bss) - KERNEL_OFFSET) ALIGN(4K) {
        *(.bss)
        *(COMMON)
    }
    _kernel_end = .;
}
