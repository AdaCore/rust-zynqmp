/*
Basic Cortex-A linker script.

You must supply a file called `memory.x` which defines the memory regions 'CODE' and 'DATA'.

The stack pointer(s) will be (near) the top of the DATA region by default.

Based upon the linker script from https://github.com/rust-embedded/cortex-ar/cortex-a-rt
*/

INCLUDE memory.x

_DEFAULT_STACK_SIZE = (8 * 1024 * 1024);

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
        . += DEFINED (__stack_size) ? __stack_size : _DEFAULT_STACK_SIZE;
        . = ALIGN(0x1000);
        __estack0 = .;

        /* Guard page */
        . = . + 0x1000;

        __sstack1 = .;
        . += DEFINED (__stack_size) ? __stack_size : _DEFAULT_STACK_SIZE;
        . = ALIGN(0x1000);
        __estack1 = .;

        /* Guard page */
        . = . + 0x1000;

        __sstack2 = .;
        . += DEFINED (__stack_size) ? __stack_size : _DEFAULT_STACK_SIZE;
        . = ALIGN(0x1000);
        __estack2 = .;

        /* Guard page */
        . = . + 0x1000;

        __sstack3 = .;
        . += DEFINED (__stack_size) ? __stack_size : _DEFAULT_STACK_SIZE;
        . = ALIGN(0x1000);
        __estack3 = .;

        __estack = .;

        /* Guard page */
        . = . + 0x1000;
    } > DATA

    /DISCARD/ : {
        *(.note .note*)
    }
}
