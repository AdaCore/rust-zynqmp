/*
Basic Cortex-A linker script.

You must supply a file called `memory.x` which defines the memory regions 'CODE' and 'DATA'.

The stack pointer(s) will be (near) the top of the DATA region by default.

Based upon the linker script from https://github.com/rust-embedded/cortex-ar/cortex-a-rt
*/

INCLUDE memory.x

_DEFAULT_STACK_SIZE = (8 * 1024 * 1024);
_DEFAULT_HEAP_SIZE = (32 * 1024 * 1024);

__stack_size = DEFINED (__stack_size) ? __stack_size : _DEFAULT_STACK_SIZE;
__heap_size = DEFINED (__heap_size) ? __heap_size : _DEFAULT_HEAP_SIZE;

SECTIONS {
    .text : {
        KEEP (*(.vectors))
        *(.boot)
        *(.text .text*)
    } > CODE

    .rodata : {
        *(.rodata .rodata*)
    } > CODE

    .data : ALIGN(8) {
        . = ALIGN(8);
        __data_start = .;
        *(.data .data.*);
        . = ALIGN(8);
    } > DATA AT>CODE
    . = ALIGN(8);
    __data_end = .;
    __data_dwords = (__data_end - __data_start) >> 3;

    __data_load_start = LOADADDR(.data);

    .bss (NOLOAD) : ALIGN(8) {
        . = ALIGN(8);
        __bss_start = .;
        *(.bss .bss* COMMON)
        . = ALIGN(8);
    } > DATA
    __bss_end = .;

    .stack (NOLOAD) : ALIGN(0x1000) {
        /* Guard page */
        . = . + 0x1000;

        __stack_start = .;

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

        __stack_end = .;

        /* Guard page */
        . = . + 0x1000;
    } > DATA

    .heap (NOLOAD) : ALIGN(0x1000) {
        __heap_start = .;
        . += __heap_size;
        . = ALIGN(0x1000);
        __heap_end = .;
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
