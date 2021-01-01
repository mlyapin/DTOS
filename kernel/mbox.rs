use crate::machine::rpi3bp::mbox;
use crate::mmio;

pub fn read_msg(channel: u32) -> Option<u32> {
    let readstat: u32 = mmio::read(mbox::reg::STATUS);
    let empty = (readstat & mbox::status::EMPTY) != 0;
    if empty {
        return None;
    }

    let memdata: u32 = mmio::read(mbox::reg::READ);
    let (ch, data) = (memdata & mbox::CHANNEL_MASK, memdata & !mbox::CHANNEL_MASK);
    match ch == channel {
        true => Some(data),
        false => None,
    }
}

pub fn write_msg(channel: u32, data: u32) {
    loop {
        let writestat: u32 = mmio::read(mbox::reg::STATUS);
        let full = (writestat & mbox::status::FULL) != 0;
        if !full {
            break;
        }
    }

    let data = data & !mbox::CHANNEL_MASK;
    let channel = channel & mbox::CHANNEL_MASK;

    mmio::write(mbox::reg::WRITE, data | channel);
}

#[cfg(test)]
mod tests {
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
            assert_eq!(buffer.0.req_resp_code, super::mbox::code::RESPONSE_OK);
            assert_ne!(buffer.0.tag.size, 0);
        };
    }
}
