pub mod testing;

pub trait Int: core::marker::Sized {}

macro_rules! impl_integer {
    ($T:ty) => {
        impl Int for $T {}
    };
}

impl_integer!(u8);
impl_integer!(u16);
impl_integer!(u32);
impl_integer!(u64);
impl_integer!(u128);
impl_integer!(usize);
impl_integer!(i8);
impl_integer!(i16);
impl_integer!(i32);
impl_integer!(i64);
impl_integer!(i128);
impl_integer!(isize);

#[inline(always)]
pub fn waste_cycles(count: usize) {
    unsafe {
        asm!(
            "1:",
            "subs {count}, {count}, #1",
            "cbnz xzr, 1b",
            count = in(reg) count
        );
    }
}
