use crate::*;

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
