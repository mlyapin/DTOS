#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![feature(naked_functions)]
#![feature(asm)]

#![test_runner(lkernel::util::testing::unit_tests_runner)]
#![reexport_test_harness_main = "tests_main"]

mod boot;

use lkernel::{
    early,
    regapi,
    kprint,
    println
};

#[no_mangle]
pub extern "C" fn kernel_init() -> ! {
    let uart = regapi::RegFile::at_addr(early::rpi3bp::UART0_BASE);
    let gpio = regapi::RegFile::at_addr(early::rpi3bp::GPIO_BASE);
    kprint::register_writer(early::uart0::Uart0::create_global(uart, gpio));

    #[cfg(test)]
    tests_main();

    println!("Successfully booted.");

    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    lkernel::util::testing::tests_panic(info);
}
