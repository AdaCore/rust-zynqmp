.text

.global __start

.set rvbar,        0xfd5c0040  // RVBARADDR0L (APU): Reset Vector Base Address
.set gicd_igroupr, 0xf9010080  // GICD_IGROUPR0 (GIC400): Interrupt Group Register
.set gicc_pmr,     0xf9020004  // GICC_PMR (GIC400): Interrupt Priority Mask Register

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

    // EL3 startup
    cmp     x1, #(3<<2)
    beq     __start_el3

    // EL2 startup
    cmp     x1, #(2<<2)
    beq     __start_el2

    // EL1 startup
    cmp     x1, #(1<<2)
    beq     __start_el1

    // EL0 startup not supported
    b       __error

.type __start_el3, %function
__start_el3:
    // Set vector base address
    ldr     x0, =__vectors
    msr     vbar_el3, x0

    // Set reset vector base address
    ldr     w1, =rvbar
    mov     w3, #0x8
    mul     w3, w3, w7  // Offset from rvbar: CPU_num * 8
    add     w1, w1, w3
    str     x0, [x1]

    // Set timer frequency (indicator for software)
    ldr     x0, =99999000
    msr     cntfrq_el0, x0

    // Set security state configuration register:
    // NS  = 1  EL0 and EL1 are in non-secure state
    // IRQ = 0  Do not take physical IRQ from EL0-2 at EL3
    // FIQ = 0  Do not take physical FIQ from EL0-2 at EL3
    // EA  = 0  Do not take External Aborts and SError Interrupts from EL0-2 at EL3
    // SMD = 0  Enable SMC instruction at EL1-3
    // HCE = 1  Enable HVC instruction at EL1-3
    // SIF = 0  Permit secure state instruction fetches from non-secure memory
    // RW  = 1  Next lower level is AArch64
    // ST  = 0  Enable access to CNTPS_*_EL1 registers only at EL3
    // TWI = 0  Do not trap WFI instructions
    // TWE = 0  Do not trap WFE instructions
    mov     x0, #0x501
    msr     scr_el3, x0

    // Set system control register:
    // M   = 0  Disable EL3 MMU
    // A   = 1  Enable alignment fault check
    // C   = 1  Enable data and unified caches
    // SA  = 0  Disable stack alignment check
    // I   = 1  Enable instruction caches
    // WXN = 0  Do not force XN for regions with write permissions
    // EE  = 0  Set exception endianness to little endian
    mov     x0, #0x1006
    msr     sctlr_el3, x0

    // Do not trap debug accesses
    msr     mdcr_el3, xzr

    // Enable write access to registers at EL2
    mov     x0, #0x73
    msr     actlr_el3, x0

    // Enable FPU (do not trap accesses)
    msr     cptr_el3, xzr

    // Set all interrupts to group 1 (non-secure interrupts)
    ldr     w2, =gicd_igroupr
    mov     w1, #0xffffffff
    str     w1, [x2]
    str     w1, [x2, #4]
    str     w1, [x2, #8]
    str     w1, [x2, #12]
    str     w1, [x2, #16]
    str     w1, [x2, #20]

    // Allow non-secure interrupt priorities
    ldr     w2, =gicc_pmr
    mov     w1, #0x80
    str     w1, [x2]

    // Switch to EL2, set AIF
    mov     x0, #(0x1c0 + 9)
    msr     spsr_el3, x0
    adr     x0, __start_el2
    msr     elr_el3, x0

    isb
    eret

.type __start_el2, %function
__start_el2:
    // Set vector base address
    ldr     x0, =__vectors
    msr     vbar_el2, x0

    // Enable FPU (do not trap accesses)
    msr     cptr_el2, xzr

    // Enable hardware management of data coherency with other cores
    msr     s3_1_c15_c2_1, x1

    // Initialize generic timers
    mrs     x0, cnthctl_el2
    orr     x0, x0, #0x3  // Enable access to timers at EL1
    msr     cnthctl_el2, x0
    msr     cntvoff_el2, xzr

    // Set Virtualization Processor ID and Virtualization Multiprocessor ID
    mrs     x0, midr_el1
    mrs     x1, mpidr_el1
    msr     vpidr_el2, x0
    msr     vmpidr_el2, x1

    // Enable AArch64 at EL1 and disable HVC
    mov     x0, #(1<<31)
    orr     x0, x0, #(1<<29)
    msr     hcr_el2, x0

    // Switch to EL1, set AIF
    mov     x0, #(0x1c0 + 5)
    msr     spsr_el2, x0
    adr     x0, __start_el1
    msr     elr_el2, x0

    isb
    eret

.type __start_el1, %function
__start_el1:
    // Set vector base address
    ldr     x0, =__vectors
    msr     vbar_el1, x0

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
