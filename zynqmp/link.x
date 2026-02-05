/*
Basic Cortex-A linker script.

You must supply a file called `memory.x` which defines the memory regions 'CODE' and 'DATA'.

The stack pointer(s) will be (near) the top of the DATA region by default.

Based upon the linker script from https://github.com/rust-embedded/cortex-ar/cortex-a-rt
*/

INCLUDE memory.x

PROVIDE_HIDDEN(__stack_size = 8M);

SECTIONS {
    .text : ALIGN(0x1000) {
        __text_start = .;
        KEEP(*(.vectors))
        *(.boot)
        *(.text)
        *(.text.*)
        __text_end = .;
    } > CODE

    .rodata : ALIGN(0x1000) {
        *(.rodata)
        *(.rodata.*)
    } > CODE

    .data : ALIGN(0x1000) {
        __data_start = .;
        *(.data)
        *(.data.*)
        . = ALIGN(8);
        __data_end = .;
    } > DATA AT>CODE

    __data_load_start = LOADADDR(.data);

    .bss (NOLOAD) : ALIGN(8) {
        __bss_start = .;
        *(.bss)
        *(.bss.*)
        *(COMMON)
        . = ALIGN(8);
        __bss_end = .;
    } > DATA

    .stack (NOLOAD) : ALIGN(0x1000) {
        __stack_start = .;

        /* Guard page */
        . = . + 0x1000;

        __stack0_start = .;
        . += __stack_size;
        . = ALIGN(0x1000);
        __stack0_end = .;

        /* Guard page */
        . = . + 0x1000;

        __stack1_start = .;
        . += __stack_size;
        . = ALIGN(0x1000);
        __stack1_end = .;

        /* Guard page */
        . = . + 0x1000;

        __stack2_start = .;
        . += __stack_size;
        . = ALIGN(0x1000);
        __stack2_end = .;

        /* Guard page */
        . = . + 0x1000;

        __stack3_start = .;
        . += __stack_size;
        . = ALIGN(0x1000);
        __stack3_end = .;

        /* Guard page */
        . = . + 0x1000;

        __stack_end = .;
    } > DATA

    .heap (NOLOAD) : ALIGN(0x1000) {
        __heap_start = .;
        __heap_end = ORIGIN(DATA) + LENGTH(DATA);
        __heap_size = __heap_end - __heap_start;
    } > DATA

    /DISCARD/ : {
        *(.note .note*)
    }
}

/* Weak aliases for default exception handlers */
PROVIDE(_sync_handler   = __default_handler);
PROVIDE(_irq_handler    = __default_handler);
PROVIDE(_fiq_handler    = __default_handler);
PROVIDE(_serror_handler = __default_handler);

/* Weak alias for default exit handler */
PROVIDE(_exit_handler   = __default_exit_handler);

/* Prerequisites for correct MMU configuration */
ASSERT(__stack_end < 0x40000000, "ERROR: stack exceeds first 1GB memory block")
