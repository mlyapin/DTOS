pub struct RegFile {
    base: usize,
}

impl RegFile {
    pub const fn at_addr(address: usize) -> Self {
        Self {
            base: address,
        }
    }

    pub fn read(&self, reg_offset: u32) -> u32 {
        let addr = self.base + reg_offset as usize;
        unsafe {
            core::ptr::read_volatile(addr as *const u32)
        }
    }

    pub fn write(&self, reg_offset: u32, value: u32) {
        let addr = self.base + reg_offset as usize;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, value);
        }
    }
}
