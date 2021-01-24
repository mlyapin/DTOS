#[inline(always)]
pub fn waste_cycles(count: usize) {
    unsafe {
        asm!(
            "1:",
            "subs {count}, {count}, #1",
            "cbnz xzr, 1b",
            count = in(reg) count
        );
    }
}
