// Usually protected by a mutex.
pub type ThreadSafeWriter = dyn core::fmt::Write + Sync + Send;

static WRITER: spin::Mutex<Option<&'static mut ThreadSafeWriter>> = spin::Mutex::new(None);

pub fn register_writer(tsw: &'static mut ThreadSafeWriter) {
    *WRITER.lock() = Some(tsw);
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    if let Some(w) = &mut *WRITER.lock() {
        w.write_fmt(args).unwrap();
    }
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n\r"));
    ($($arg:tt)*) => ($crate::print!("{}\n\r", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::kprint::_print(format_args!($($arg)*)));
}
