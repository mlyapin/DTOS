use crate::util;

// Linux's virtual address is 0x7E000000;
// Physical address is 0x3F000000;
pub const MMIO_BASE: usize = 0x3F000000;

pub mod gpio {
    const BASE: usize = super::MMIO_BASE + 0x200000;

    pub mod reg {
        use super::BASE;
        pub const GPPUD: usize = BASE + 0x94;
        pub const GPPUD_CLK0: usize = BASE + 0x98;
    }
}

pub mod uart0 {
    // See BCM2837 Arm Peripherals datasheet for other registers.
    const BASE: usize = super::MMIO_BASE + 0x201000;

    pub mod reg {
        use super::BASE;
        pub const DR: usize = BASE + 0x00;
        pub const RSRECR: usize = BASE + 0x04;
        pub const FR: usize = BASE + 0x18;
        pub const IBRD: usize = BASE + 0x24;
        pub const FBRD: usize = BASE + 0x28;
        pub const LCRH: usize = BASE + 0x2C;
        pub const CR: usize = BASE + 0x30;
        pub const IFLS: usize = BASE + 0x34;
        pub const IMSC: usize = BASE + 0x38;
        pub const RIS: usize = BASE + 0x3C;
        pub const MIS: usize = BASE + 0x40;
    }
}

pub mod mbox {
    const BASE: usize = super::MMIO_BASE + 0xB880;

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

const VALUE_BUFFER_LEN: usize = 0x10;
#[repr(C, packed)]
struct Msg<T: util::Int> {
    buffer_len: u32,
    reqresp_code: u32,
    tag_id: u32,
    request_value_len: u32,
    tag_reqresp_code: u32,
    value: [T; VALUE_BUFFER_LEN],
    must_be_zero: u32,
}

#[repr(C, align(16))]
struct AlignedMsg<T: util::Int>(Msg<T>);

fn send_mail(
    channel: u32,
    tag: u32,
    payload_buffer: &mut [u32],
    data_len: &mut u32,
) -> Result<(), &'static str> {
    let mut msg: AlignedMsg<u32> = AlignedMsg(Msg {
        buffer_len: core::mem::size_of::<Msg<u32>>() as u32,
        reqresp_code: mbox::code::REQUEST,
        tag_id: tag,
        request_value_len: *data_len,
        tag_reqresp_code: 0,
        value: [0; VALUE_BUFFER_LEN],
        must_be_zero: 0,
    });
    unsafe {
        msg.0.value.copy_from_slice(payload_buffer);
    }

    crate::mbox::write_msg(channel, &mut msg as *mut AlignedMsg<u32> as u32);
    let _ = crate::mbox::read_msg(channel);

    match msg.0.reqresp_code {
        mbox::code::RESPONSE_OK => {
            *data_len = msg.0.tag_reqresp_code & mbox::RESPONSE_LEN_MASK;
            unsafe {
                payload_buffer.copy_from_slice(&msg.0.value);
            }
            Ok(())
        }
        mbox::code::RESPONSE_ERR => Err("Error parsing request buffer"),
        mbox::code::REQUEST => Err("Missing response from the VC"),
        _ => Err("Unexpected response code"),
    }
}

pub fn set_clock_rate(clock_id: u32, rate: u32, skip_turbo: bool) -> Result<u32, &'static str> {
    const EXPECTED_RESP_LEN: u32 = 8;

    let mut payload_buffer = [0 as u32; VALUE_BUFFER_LEN];
    payload_buffer[0] = clock_id;
    payload_buffer[1] = rate;
    payload_buffer[2] = if skip_turbo { 1 } else { 0 };
    let mut actual_len = core::mem::size_of_val(&payload_buffer[0..3]) as u32;

    send_mail(
        mbox::channel::PROPERTY_TAGS_VC,
        mbox::tag::SET_CLOCK_RATE,
        &mut payload_buffer,
        &mut actual_len,
    )
    .unwrap_or_else(|e| panic!(e));

    if actual_len != EXPECTED_RESP_LEN {
        Err("Unexpected response length")
    } else if payload_buffer[1] == 0 {
        Err("The clock does not exist")
    } else {
        Ok(payload_buffer[1])
    }
}

pub fn get_clock_rate(clock_id: u32) -> Result<u32, &'static str> {
    const EXPECTED_RESP_LEN: u32 = 8;

    let mut payload_buffer = [0 as u32; VALUE_BUFFER_LEN];
    payload_buffer[0] = clock_id;
    let mut actual_len = core::mem::size_of_val(&payload_buffer[0..1]) as u32;

    send_mail(
        mbox::channel::PROPERTY_TAGS_VC,
        mbox::tag::GET_CLOCK_RATE,
        &mut payload_buffer,
        &mut actual_len,
    )
    .unwrap_or_else(|e| panic!(e));

    if actual_len != EXPECTED_RESP_LEN {
        Err("Unexpected response length")
    } else if payload_buffer[1] == 0 {
        Err("The clock does not exist")
    } else {
        Ok(payload_buffer[1])
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
