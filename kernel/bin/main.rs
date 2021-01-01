#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lkernel::util::testing::unit_tests_runner)]
#![reexport_test_harness_main = "tests_main"]

use lkernel::early_print;
use lkernel::early_println;
use lkernel::early::uart0;

#[no_mangle]
pub extern "C" fn kernel_init() -> ! {
    uart0::GLOBAL.lock().init();

    #[cfg(test)]
    tests_main();

    early_print!(">> ");

    loop {
        let c = uart0::GLOBAL.lock().getc();
        if c == b'\r' {
            early_println!();
            early_print!(">> ");
        } else {
            uart0::GLOBAL.lock().putc(c);
        }
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    early_println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    lkernel::util::testing::tests_panic(info);
}
