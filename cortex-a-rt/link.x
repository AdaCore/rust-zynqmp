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
        __sdata = .;
        *(.data .data.*);
        . = ALIGN(8);
    } > DATA AT>CODE
    . = ALIGN(8);
    __edata = .;
    __data_dwords = (__edata - __sdata) >> 3;

    __sidata = LOADADDR(.data);

    .bss (NOLOAD) : ALIGN(8) {
        . = ALIGN(8);
        __sbss = .;
        *(.bss .bss* COMMON)
        . = ALIGN(8);
    } > DATA
    __ebss = .;

    .stack (NOLOAD) : ALIGN(0x1000) {
        /* Guard page */
        . = . + 0x1000;
        __sstack = .;

        __sstack0 = .;
        . += __stack_size;
        . = ALIGN(0x1000);
        __estack0 = .;

        /* Guard page */
        . = . + 0x1000;

        __sstack1 = .;
        . += __stack_size;
        . = ALIGN(0x1000);
        __estack1 = .;

        /* Guard page */
        . = . + 0x1000;

        __sstack2 = .;
        . += __stack_size;
        . = ALIGN(0x1000);
        __estack2 = .;

        /* Guard page */
        . = . + 0x1000;

        __sstack3 = .;
        . += __stack_size;
        . = ALIGN(0x1000);
        __estack3 = .;

        __estack = .;

        /* Guard page */
        . = . + 0x1000;
    } > DATA

    .heap (NOLOAD) : ALIGN(0x1000) {
        __sheap = .;
        . += __heap_size;
        . = ALIGN(0x1000);
        __eheap = .;
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
