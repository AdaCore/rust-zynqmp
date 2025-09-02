.org 0
.text

.global __vectors
.global __start
.global __sync_interrupt
.global __irq_interrupt
.global __fiq_interrupt
.global __serror_interrupt

.macro save_general_purpose_registers
        stp     x0, x1, [sp, #-176]!
        stp     x2, x3, [sp, #16]
        stp     x4, x5, [sp, #32]
        stp     x6, x7, [sp, #48]
        stp     x8, x9, [sp, #64]
        stp     x10, x11, [sp, #80]
        stp     x12, x13, [sp, #96]
        stp     x14, x15, [sp, #112]
        stp     x16, x17, [sp, #128]
        stp     x18, x19, [sp, #144]
        stp     x29, x30, [sp, #160]
.endm

.macro restore_general_purpose_registers
        ldp     x2, x3, [sp, #16]
        ldp     x4, x5, [sp, #32]
        ldp     x6, x7, [sp, #48]
        ldp     x8, x9, [sp, #64]
        ldp     x10, x11, [sp, #80]
        ldp     x12, x13, [sp, #96]
        ldp     x14, x15, [sp, #112]
        ldp     x16, x17, [sp, #128]
        ldp     x18, x19, [sp, #144]
        ldp     x29, x30, [sp, #160]
        ldp     x0, x1, [sp], #176
.endm

.macro save_system_registers_el1
        mrs     x0, spsr_el1
        mrs     x1, elr_el1
        mrs     x2, cpacr_el1
        stp     x0, x1, [sp, #-16]!
        str     x2, [sp, #-16]!
.endm

.macro restore_system_registers_el1
        ldr     x2, [sp], #16
        ldp     x0, x1, [sp], #16
        msr     spsr_el1, x0
        msr     elr_el1, x1
        msr     cpacr_el1, x2
.endm

.section .vectors, "a"

__vectors:
.set vbar, __vectors

.org vbar
        b       __start
.org (vbar + 0x200)
        b       __sync_interrupt
.org (vbar + 0x280)
        b       __irq_interrupt
.org (vbar + 0x300)
        b       __fiq_interrupt
.org (vbar + 0x380)
        b       __serror_interrupt

.text

.type __sync_interrupt, %function
__sync_interrupt:
        save_general_purpose_registers

        bl      _sync_handler

        restore_general_purpose_registers

        eret

.type __irq_interrupt, %function
__irq_interrupt:
        save_general_purpose_registers
        save_system_registers_el1

        bl      _irq_handler

        restore_system_registers_el1
        restore_general_purpose_registers

        eret

.type __fiq_interrupt, %function
__fiq_interrupt:
        save_general_purpose_registers
        save_system_registers_el1

        bl      _fiq_handler

        restore_system_registers_el1
        restore_general_purpose_registers

        eret

.type __serror_interrupt, %function
__serror_interrupt:
        save_general_purpose_registers

        bl      _serror_handler

        restore_general_purpose_registers

        eret
