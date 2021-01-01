use lkernel::util::testing::{
    qemu_exit_ok,
    qemu_exit_err
};

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    lkernel::early_println!("[ok]");
    qemu_exit_ok();
}

#[no_mangle]
fn _unit_tests_on_test_pass() {
    lkernel::early_println!("[did not panic]");
    qemu_exit_err();
}
