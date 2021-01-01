use crate::machine::rpi3bp::{self, gpio, uart0};
use crate::mmio;
use crate::util::waste_cycles;

use core::fmt;

use spin::Mutex;

pub static GLOBAL: Mutex<UART0> = Mutex::new(UART0 {});

pub struct UART0;

impl UART0 {
    pub fn init(&self) {
        // Required actions are described in the BCM2837 datasheet.

        // Disable UART0.
        mmio::write(uart0::reg::CR, 0x0 as u32);

        // Disable pull up/down for all GPIO pins.
        mmio::write(gpio::reg::GPPUD, 0x0 as u32);
        waste_cycles(150);

        mmio::write(gpio::reg::GPPUD_CLK0, ((0x1 << 14) | (0x1 << 15)) as u32);
        waste_cycles(150);

        // Confirm changes by writing to GPPUD again.
        mmio::write(gpio::reg::GPPUD, 0x0 as u32);

        let clock_rate = 3000000;

        let actual_rate = rpi3bp::set_clock_rate(rpi3bp::mbox::clock::UART,
                                                 clock_rate,
                                                 false)
            .unwrap_or_else(|e| panic!(e));

        if actual_rate != clock_rate {
            panic!("Can't set required UART0 clock rate");
        }

        // Whole part of clock_rate / (16 * 115200) = 1.627 = 1
        mmio::write(uart0::reg::IBRD, 1);
        // Uhmm.. Don't understand this part, to be honest.
        // Copied from: https://wiki.osdev.org/Raspberry_Pi_Bare_Bones
        mmio::write(uart0::reg::FBRD, 40);

        let enable_fifo = 0x1 << 4;
        let wordlen_8 = 0x1 << 5 | 0x1 << 6;
        mmio::write(uart0::reg::LCRH, enable_fifo | wordlen_8);

        let enable_uart = 0x1 << 0;
        let enable_tx = 0x1 << 8;
        let enable_rx = 0x1 << 9;
        mmio::write(uart0::reg::CR, enable_uart | enable_tx | enable_rx);
    }

    fn wait_for_data(&self) {
        loop {
            let uart_flags: u32 = mmio::read(uart0::reg::FR);
            let recv_fifo_empty = (uart_flags & (0x1 << 4)) != 0;
            if !recv_fifo_empty {
                break;
            }
        }
    }

    fn wait_for_space(&self) {
        loop {
            let uart_flags: u32 = mmio::read(uart0::reg::FR);
            let send_fifo_full = (uart_flags & (0x1 << 5)) != 0;
            if !send_fifo_full {
                break;
            }
        }
    }

    pub fn putc(&self, c: u8) {
        self.wait_for_space();
        mmio::write(uart0::reg::DR, c);
    }

    pub fn getc(&self) -> u8 {
        self.wait_for_data();
        let data: u32 = mmio::read(uart0::reg::DR);
        (data & 0xFF) as u8
    }

    pub fn puts(&self, s: &str) {
        for b in s.bytes() {
            self.putc(b);
        }
    }
}

impl fmt::Write for UART0 {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.puts(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! early_println {
    () => ($crate::early_print!("\n\r"));
    ($($arg:tt)*) => ($crate::early_print!("{}\n\r", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! early_print {
    ($($arg:tt)*) => ($crate::early::uart0::_print(format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    GLOBAL
        .lock()
        .write_fmt(args)
        .expect("Couldn't write to UART0");
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn early_print_works() {
        early_print!("It works ");
        early_print!("indeed...\t");
    }
}
