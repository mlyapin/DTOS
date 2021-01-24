#![no_std]
#![cfg_attr(test, no_main)]
#![test_runner(util::testing::unit_tests_runner)]
#![reexport_test_harness_main = "tests_main"]

#![feature(custom_test_frameworks)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(linkage)]

pub mod boot;
pub mod util;
pub mod early;
pub mod sync;
pub mod regapi;
pub mod kprint;

#[cfg(test)]
#[no_mangle]
pub extern "C" fn kernel_init() -> ! {
    let uart = regapi::RegFile::at_addr(early::rpi3bp::UART0_BASE);
    let gpio = regapi::RegFile::at_addr(early::rpi3bp::GPIO_BASE);
    kprint::register_writer(early::uart0::Uart0::create_global(uart, gpio));

    tests_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    util::testing::tests_panic(info);
}
