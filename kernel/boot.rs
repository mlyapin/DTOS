#[link_section = ".boot.text"]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn _start() {
    extern "C" {
        static __bootstack_bottom: *const core::ffi::c_void;
        static __bss_start: *const core::ffi::c_void;
        static __bss_end: *const core::ffi::c_void;
        fn kernel_init() -> !;
    }

    asm!(
        // Hang if we're not on the first cpu.
        "mrs x1, mpidr_el1",
        "and x1, x1, #3",
        "cbz x1, 2f",
        "1:",
        "wfe",
        "b 1b",
        "2:",
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
        // Begin the execution.
        "b {kernel_init}",
        stack_bottom = sym __bootstack_bottom,
        kernel_init = sym kernel_init,
        start = sym __bss_start,
        end = sym __bss_end,
        options(noreturn)
    );
}
