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
pub mod machine;
pub mod mmio;
pub mod mbox;
pub mod early;

#[cfg(test)]
#[no_mangle]
pub extern "C" fn kernel_init() -> ! {
    early::uart0::GLOBAL.lock().init();

    tests_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    util::testing::tests_panic(info);
}
