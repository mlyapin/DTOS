
unsafe fn start_kernel() -> ! {
    extern "C" {
        static __bootstack_bottom: *const core::ffi::c_void;
        static __bss_start: *const core::ffi::c_void;
        static __bss_end: *const core::ffi::c_void;
        fn kernel_init() -> !;
    }

    asm!(
        // Setup the sp register.
        "ldr x5, ={stack_bottom}",
        "mov sp, x5",
        // Clear bss.
        "ldr x5, ={end}",
        "ldr x6, ={start}",
        "sub x5, x5, x6",
        "cbz x5, 4f",
        "3:",
        "str xzr, [x6], #8",
        "sub x5, x5, #1",
        "cbnz x5, 3b",
        "4:",
        "b {init}",
        stack_bottom = sym __bootstack_bottom,
        init = sym kernel_init,
        start = sym __bss_start,
        end = sym __bss_end,
        options(noreturn)
    );
}

unsafe fn switch_to_el1() -> ! {
    // Couldn't find the list of implemented features by BCM2710.
    // Copied from https://github.com/s-matyukevich/raspberry-pi-os/blob/master/docs/lesson02/rpi-os.md
    const SCTLR_RES1: u64 = (1 << 29) | (1 << 28) | (1 << 23) | (1 << 22) | (1 << 20) | (1 << 11);
    const SCTLR_ENDIANNESS: u64 = /* EL1 LE */ (0 << 25) | /* EL0 LE */ (0 << 24);

    // Use AArch64 at EL1.
    const HCR_RW: u64 = 1 << 31;

    const SCR_RES1: u64 = 3 << 4;
    const SCR_NS: u64 = 1 << 0;
    // Use AArch64 at next EL(2).
    const SCR_RW: u64 = 1 << 10;

    const SPSR_INT_MASK: u64 = 7 << 6;
    const SPSR_EL1H: u64 = 5 << 0;

    asm!(
        "msr sctlr_el1, {sctlr:x}",
        "msr hcr_el2, {hcr:x}",
        "msr scr_el3, {scr:x}",
        "msr spsr_el3, {spsr:x}",
        "adr {fnaddr:x}, {next_fn}",
        "msr elr_el3, {fnaddr:x}",
        "eret",
        sctlr = in(reg) SCTLR_RES1 | SCTLR_ENDIANNESS,
        hcr = in(reg) HCR_RW,
        scr = in(reg) SCR_RES1 | SCR_NS | SCR_RW,
        spsr = in(reg) SPSR_INT_MASK | SPSR_EL1H,
        next_fn = sym start_kernel,
        fnaddr = in(reg) 0,
        options(noreturn)
    );
}

#[link_section = ".boot.text"]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn _start() -> ! {

    asm!(
        // Hang if we're not on the first cpu.
        "mrs x1, mpidr_el1",
        "and x1, x1, #3",
        "cbz x1, 2f",
        "1:",
        "wfe",
        "b 1b",
        "2:",
        "b {next_fn}",
        next_fn = sym switch_to_el1,
        options(noreturn)
    );
}
