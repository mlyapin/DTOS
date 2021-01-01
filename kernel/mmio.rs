use crate::util;

pub fn write<T: util::Int>(reg: usize, val: T) {
    unsafe { core::ptr::write_volatile(reg as *mut T, val) }
}

pub fn read<T: util::Int>(reg: usize) -> T {
    unsafe { core::ptr::read_volatile(reg as *const T) }
}
