#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lkernel::util::testing::unit_tests_runner)]
#![reexport_test_harness_main = "tests_main"]

mod must_not_panic;

#[no_mangle]
pub extern "C" fn kernel_init() -> ! {
    lkernel::early::uart0::GLOBAL.lock().init();
    tests_main();
    loop {}
}

#[test_case]
fn pass() {
    assert_eq!(1, 1);
}
