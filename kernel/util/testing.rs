use crate::{print, println};

pub fn unit_tests_runner(tests: &[&dyn Testable]) {
    println!("Running {} tests", tests.len());
    for t in tests {
        t.run();
    }
    qemu_exit_ok();
}


pub fn tests_panic(info: &core::panic::PanicInfo) -> ! {
    println!("[failed]");
    println!("Error: {}", info);
    qemu_exit_err();
}

pub fn qemu_exit_ok() -> ! {
    use qemu_exit::QEMUExit;

    #[cfg(target_arch = "aarch64")]
    let exit_handle = qemu_exit::AArch64::new();

    exit_handle.exit_success();
}

pub fn qemu_exit_err() -> ! {
    use qemu_exit::QEMUExit;

    #[cfg(target_arch = "aarch64")]
    let exit_handle = qemu_exit::AArch64::new();

    exit_handle.exit_failure();
}

pub trait Testable {
    fn run(&self);
}

impl<T: Fn()> Testable for T {
    fn run(&self) {
        print!("{}...\t", core::any::type_name::<T>());
        self();
        _unit_tests_on_test_pass();
    }
}

#[linkage = "weak"]
#[no_mangle]
fn _unit_tests_on_test_pass() {
        println!("[ok]");
}
