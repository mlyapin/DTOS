#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    lkernel::util::testing::tests_panic(info);
}
