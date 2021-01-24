pub const MMIO_BASE: usize = 0x3F000000;
pub const UART0_BASE: usize = MMIO_BASE + 0x201000;
pub const GPIO_BASE: usize = MMIO_BASE + 0x200000;

pub mod mbox {
    pub const BASE: usize = super::MMIO_BASE + 0xB880;

    pub const CHANNEL_MASK: u32 = 0xF;
    pub const RESPONSE_LEN_MASK: u32 = 0x7FFFFFFF;

    pub mod reg {
        use super::BASE;

        pub const READ: usize = BASE + 0x00;
        pub const STATUS: usize = BASE + 0x18;
        pub const WRITE: usize = BASE + 0x20;
    }

    pub mod code {
        pub const RESPONSE_OK: u32 = 0x80000000;
        pub const RESPONSE_ERR: u32 = 0x80000001;
        pub const REQUEST: u32 = 0x0;
    }

    pub mod status {
        pub const EMPTY: u32 = 0x1 << 30;
        pub const FULL: u32 = 0x1 << 31;
    }

    pub mod tag {
        pub const GET_CLOCK_STATE: u32 = 0x30001;
        pub const SET_CLOCK_STATE: u32 = 0x38001;
        pub const GET_CLOCK_RATE: u32 = 0x30002;
        pub const SET_CLOCK_RATE: u32 = 0x38002;
    }

    pub mod channel {
        pub const PROPERTY_TAGS_VC: u32 = 8;
    }

    pub mod clock {
        pub const UART: u32 = 0x2;
        pub const CORE: u32 = 0x4;
    }
}

use crate::{mbox_pickup_mail, mbox_send_mail};

pub fn set_clock_rate(clock_id: u32, rate: u32, skip_turbo: bool) -> Result<u32, &'static str> {
    let mail = mbox_send_mail!(
        mbox::channel::PROPERTY_TAGS_VC,
        mbox::tag::SET_CLOCK_RATE,
        clock_id,
        rate,
        if skip_turbo { 1 } else { 0 }
    );

    let rate;
    let len = mbox_pickup_mail!(mail, _, rate);
    const EXPECTED_RESP_LEN: u32 = 8;

    if len != EXPECTED_RESP_LEN {
        Err("Unexpected response length")
    } else if rate == 0 {
        Err("The clock does not exist")
    } else {
        Ok(rate)
    }
}

pub fn get_clock_rate(clock_id: u32) -> Result<u32, &'static str> {
    let mail = mbox_send_mail!(
        mbox::channel::PROPERTY_TAGS_VC,
        mbox::tag::GET_CLOCK_RATE,
        clock_id
    );

    let rate;
    let len = mbox_pickup_mail!(mail, _, rate);
    const EXPECTED_RESP_LEN: u32 = 8;

    if len != EXPECTED_RESP_LEN {
        Err("Unexpected response length")
    } else if rate == 0 {
        Err("The clock does not exist")
    } else {
        Ok(rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const CLOCK_ID: u32 = mbox::clock::UART;

    #[test_case]
    fn can_read_clockrates() {
        let clock_rate = get_clock_rate(CLOCK_ID);
        assert!(clock_rate.is_ok());
    }

    #[test_case]
    fn can_change_clockrates() {
        const NEW_CLOCKRATE: u32 = 460800;
        let previous_clockrate = get_clock_rate(CLOCK_ID);
        assert_ne!(previous_clockrate, Ok(NEW_CLOCKRATE));

        let set_clockrate = set_clock_rate(CLOCK_ID, NEW_CLOCKRATE, false);
        assert_eq!(set_clockrate, Ok(NEW_CLOCKRATE));
    }
}
