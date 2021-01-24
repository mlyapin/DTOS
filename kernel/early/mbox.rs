use super::rpi3bp::{self, mbox};
use crate::regapi;
use crate::util::constraints;

static MBOX_FILE: regapi::RegFile = regapi::RegFile::at_addr(rpi3bp::mbox::BASE);

// Mbox register offsets
const READ: u32 = 0x00;
const STATUS: u32 = 0x18;
const WRITE: u32 = 0x20;

pub fn read_msg(channel: u32) -> Option<u32> {
    let readstat = MBOX_FILE.read(STATUS);
    let empty = (readstat & mbox::status::EMPTY) != 0;
    if empty {
        return None;
    }

    let memdata: u32 = MBOX_FILE.read(READ);
    let (ch, data) = (memdata & mbox::CHANNEL_MASK, memdata & !mbox::CHANNEL_MASK);
    match ch == channel {
        true => Some(data),
        false => None,
    }
}

pub fn write_msg(channel: u32, data: u32) {
    loop {
        let writestat: u32 = MBOX_FILE.read(STATUS);
        let full = (writestat & mbox::status::FULL) != 0;
        if !full {
            break;
        }
    }

    let data = data & !mbox::CHANNEL_MASK;
    let channel = channel & mbox::CHANNEL_MASK;

    MBOX_FILE.write(WRITE, data | channel);
}

pub const VALUE_BUFFER_LEN: usize = 0x10;
#[repr(C, packed)]
struct Msg<T: constraints::Int> {
    buffer_len: u32,
    reqresp_code: u32,
    tag_id: u32,
    request_value_len: u32,
    tag_reqresp_code: u32,
    value: [T; VALUE_BUFFER_LEN],
    must_be_zero: u32,
}

#[repr(C, align(16))]
struct AlignedMsg<T: constraints::Int>(Msg<T>);

pub fn __send_mail(
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

    write_msg(channel, &mut msg as *mut AlignedMsg<u32> as u32);
    let _ = read_msg(channel);

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

pub struct SendMailPayload {
    pub buffer: [u32; VALUE_BUFFER_LEN],
    pub len: u32,
}

#[macro_export]
macro_rules! mbox_send_mail {
    ($channel:expr, $tag:expr, $($arg:expr),*) => {
        {
            let mut payload: $crate::early::mbox::SendMailPayload = $crate::early::mbox::SendMailPayload {
                buffer: [0 as u32; $crate::early::mbox::VALUE_BUFFER_LEN],
                len: 0,
            };

            let mut it = payload.buffer.iter_mut();
            $(
                *it.next().unwrap() = $arg;
                payload.len += core::mem::size_of_val(&$arg) as u32;
            )*

            $crate::early::mbox::__send_mail(
                $channel,
                $tag,
                &mut payload.buffer,
                &mut payload.len)
            .unwrap();
            payload
        }
    }
}

#[macro_export]
macro_rules! mbox_pickup_mail {
    (@rec $iter:ident, $arg:ident) => {
        $arg = $iter.next().unwrap().clone();
    };
    (@rec $iter:ident, _ ) => {
        $iter.next().unwrap();
    };
    (@rec $iter:ident, _, $($tail:tt)*) => {
        $crate::mbox_pickup_mail!(@rec $iter, _);
        $crate::mbox_pickup_mail!(@rec $iter, $($tail)*);
    };
    (@rec $iter:ident, $arg:ident, $($tail:tt)*) => {
        $crate::mbox_pickup_mail!(@rec $iter, $arg);
        $crate::mbox_pickup_mail!(@rec $iter, $($tail)*);
    };

    ($payload:expr, $($args:tt)*) => {
        {
            let mut it = $payload.buffer.iter();
            $crate::mbox_pickup_mail!(@rec it, $($args)*);
            $payload.len
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn can_read_write_mailbox() {
        #[repr(C, packed)]
        struct TagPacked {
            id: u32,
            value_len: u32,
            req_resp_codes: u32,
            base: u32,
            size: u32,
        }

        #[repr(C, packed)]
        struct BufferPacked {
            buffer_size: u32,
            req_resp_code: u32,
            tag: TagPacked,
            must_be_zero: u32,
        }
        #[repr(C, align(16))]
        struct BufferAligned(BufferPacked);

        let mut buffer = BufferAligned(BufferPacked {
            buffer_size: 32,
            req_resp_code: super::mbox::code::REQUEST,
            tag: TagPacked {
                id: 0x00010005, // Get ARM memory
                value_len: 0,
                req_resp_codes: 0,
                base: 0,
                size: 0,
            },
            must_be_zero: 0,
        });

        let buffer_addr = &mut buffer as *mut BufferAligned as u32;
        super::write_msg(8, buffer_addr);

        let resp = super::read_msg(8);
        assert_eq!(resp, Some(buffer_addr));

        unsafe {
            assert_eq!(buffer.0.req_resp_code, mbox::code::RESPONSE_OK);
            assert_ne!(buffer.0.tag.size, 0);
        };
    }
}
