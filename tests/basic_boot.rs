#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lkernel::util::testing::unit_tests_runner)]
#![reexport_test_harness_main = "tests_main"]

mod must_not_panic;

use lkernel::{
    regapi,
    early,
    kprint
};

#[no_mangle]
pub extern "C" fn kernel_init() -> ! {
    let uart = regapi::RegFile::at_addr(early::rpi3bp::UART0_BASE);
    let gpio = regapi::RegFile::at_addr(early::rpi3bp::GPIO_BASE);
    kprint::register_writer(early::uart0::Uart0::create_global(uart, gpio));
    tests_main();
    loop {}
}

#[test_case]
fn pass() {
    assert_eq!(1, 1);
}
