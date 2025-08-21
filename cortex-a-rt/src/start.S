.org 0
.text

.global __start
.global __vectors

.section .vectors, "a"

__vectors:
    b       __start

.section .boot, "ax"

.type   __start, %function
__start:
    // Zero all GP registers
    mov     x0, #0
    mov     x1, #0
    mov     x2, #0
    mov     x3, #0
    mov     x4, #0
    mov     x5, #0
    mov     x6, #0
    mov     x7, #0
    mov     x8, #0
    mov     x9, #0
    mov     x10, #0
    mov     x11, #0
    mov     x12, #0
    mov     x13, #0
    mov     x14, #0
    mov     x15, #0
    mov     x16, #0
    mov     x17, #0
    mov     x18, #0
    mov     x19, #0
    mov     x20, #0
    mov     x21, #0
    mov     x22, #0
    mov     x23, #0
    mov     x24, #0
    mov     x25, #0
    mov     x26, #0
    mov     x27, #0
    mov     x28, #0
    mov     x29, #0
    mov     x30, #0

    // Get processor ID
    mrs     x7, mpidr_el1
    and     x7, x7, #3

    // Initialize memory if processor ID == 0
    cbz     x7, __start_memory_init

    // Only single core supported
    b       __error

__start_memory_init:
    // Copy .data
    ldr     x0, = __sdata
    ldr     x1, = __edata
    ldr     x2, = __sidata
0:
    cmp     x0, x1
    b.eq    1f
    ldr     x3, [x2], #8
    str     x3, [x0], #8
    b       0b
1:

    // Clear .bss
    ldr     x0, = __sbss
    ldr     x1, = __ebss
    mov     x2, #0
0:
    cmp     x0, x1
    b.eq    1f
    str     x2, [x0], #8
    b       0b
1:

    b       __start_el

.type __start_el, %function
__start_el:
    // Check current EL
    mrs     x1, currentel

    // EL3 startup not supported
    cmp     x1, #(3<<2)
    beq     __error

    // EL2 startup not supported
    cmp     x1, #(2<<2)
    beq     __error

    // EL1 startup
    cmp     x1, #(1<<2)
    beq     __start_el1

    // EL0 startup not supported
    b       __error

.type __start_el1, %function
__start_el1:
    // Initialize stack pointer
    adrp    x2, __system_stack_base
    add     x2, x2, #:lo12:__system_stack_base
    ldr     w0, [x2, x7, lsl #2]
    mov     sp, x0

    b       __start_rust

.type __error, %function
__error:
    b       __error

.type __system_stack_base, %object
__system_stack_base:
    .word   __estack0
    .word   __estack1
    .word   __estack2
    .word   __estack3
