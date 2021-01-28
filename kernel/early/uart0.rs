use super::rpi3bp;
use crate::regapi;
use crate::sync::waste_cycles;
use crate::kprint;

use core::fmt;

pub struct Uart0 {
    uart: regapi::RegFile,
    gpio: regapi::RegFile,
}
pub struct MxUart0(spin::Mutex<Option<Uart0>>);
static mut GLOBAL_UART0: MxUart0 = MxUart0(spin::Mutex::new(None));

// GPIO register 0ffsets
const GPPUD: u32 = 0x94;
const GPPUD_CLK0: u32 = 0x98;

// UART register offsets
const DR: u32 = 0x00;
const FR: u32 = 0x18;
const IBRD: u32 = 0x24;
const FBRD: u32 = 0x28;
const LCRH: u32 = 0x2C;
const CR: u32 = 0x30;

impl Uart0 {
    pub fn create_global(
        uart: regapi::RegFile,
        gpio: regapi::RegFile,
    ) -> &'static mut kprint::ThreadSafeWriter {
        let uart0 = Uart0 { uart, gpio };

        // Disable UART0.
        uart0.uart.write(CR, 0);

        // Disable pull up/down for all GPIO pins.
        uart0.gpio.write(GPPUD, 0);
        waste_cycles(150);

        uart0.gpio.write(GPPUD_CLK0, (1 << 14) | (1 << 15));
        waste_cycles(150);

        // Confirm changes by writing to GPPUD again.
        uart0.gpio.write(GPPUD, 0);

        let clock_rate = 3000000;

        let actual_rate = rpi3bp::set_clock_rate(rpi3bp::mbox::clock::UART, clock_rate, false)
            .unwrap_or_else(|e| panic!(e));

        if actual_rate != clock_rate {
            panic!("Can't set required UART0 clock rate");
        }

        // Whole part of clock_rate / (16 * 115200) = 1.627 = 1
        uart0.uart.write(IBRD, 1);

        uart0.uart.write(FBRD, 40);

        let enable_fifo = 0x1 << 4;
        let wordlen_8 = 0x1 << 5 | 0x1 << 6;
        uart0.uart.write(LCRH, enable_fifo | wordlen_8);

        let enable_uart = 0x1 << 0;
        let enable_tx = 0x1 << 8;
        let enable_rx = 0x1 << 9;
        uart0.uart.write(CR, enable_uart | enable_tx | enable_rx);

        unsafe {
            *GLOBAL_UART0.0.lock() = Some(uart0);
            &mut GLOBAL_UART0
        }
    }

    fn wait_for_data(&self) {
        loop {
            let uart_flags: u32 = self.uart.read(FR);
            let recv_fifo_empty = (uart_flags & (0x1 << 4)) != 0;
            if !recv_fifo_empty {
                break;
            }
        }
    }

    fn wait_for_space(&self) {
        loop {
            let uart_flags: u32 = self.uart.read(FR);
            let send_fifo_full = (uart_flags & (0x1 << 5)) != 0;
            if !send_fifo_full {
                break;
            }
        }
    }

    fn putc(&self, c: u8) {
        self.wait_for_space();
        self.uart.write(DR, c as u32);
    }

    pub fn getc(&self) -> u8 {
        self.wait_for_data();
        let data: u32 = self.uart.read(DR);
        (data & 0xFF) as u8
    }

    pub fn puts(&self, s: &str) {
        for b in s.bytes() {
            self.putc(b);
        }
    }
}

impl fmt::Write for MxUart0 {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if let Some(writer) = &mut *self.0.lock() {
            writer.puts(s);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn early_print_works() {
        let uart = regapi::RegFile::at_addr(rpi3bp::UART0_BASE);
        let gpio = regapi::RegFile::at_addr(rpi3bp::GPIO_BASE);
        let writer = Uart0::create_global(uart, gpio);
        writer.write_str("It works ").unwrap();
        writer.write_str("indeed...\t").unwrap();
    }
}
