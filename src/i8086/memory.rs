use std::mem;

pub struct Memory {
    pub ram: [u8; 1048576]
}

/// Wraps the address to 0xFFFFF.
fn wrap_address(address: usize) -> usize {
    address % 0xFFFFF
}

impl Memory {
    /// Reads the memory as word.
    pub fn read_word(&self, address: usize) -> u16 {
        if address >= 0xFFFFF {
            // TODO Disable this if A20 line is off?
            unsafe {
                u16::from_le(mem::transmute::<[u8; 2], u16>([
                    self.ram[wrap_address(address)],
                    self.ram[wrap_address(address + 1)]
                ]))
            }
        } else {
            unsafe {
                let ptr = (&self.ram[address] as *const u8) as *const u16;
                u16::from_le(*ptr)
            }
        }
    }
    /// Writes the memory as word.
    pub fn write_word(&mut self, address: usize, value: u16) {
        let value_le = value.to_le();
        if address >= 0xFFFFF {
            // TODO Disable this if A20 line is off?
            unsafe {
                let value_arr = mem::transmute::<u16, [u8; 2]>(value_le);
                self.ram[wrap_address(address)] = value_arr[0];
                self.ram[wrap_address(address + 1)] = value_arr[1];
            }
        } else {
            // This feels so scary
            unsafe {
                let ptr = (&mut self.ram[address] as *mut u8) as *mut u16;
                *ptr = value_le;
            }
        }
    }
    /// Returns reference of the memory.
    pub fn get_byte(&self, address: usize) -> &u8 {
        &self.ram[address]
    }
    /// Returns mutable reference of the memory.
    pub fn get_byte_mut(&mut self, address: usize) -> &mut u8 {
        &mut self.ram[address]
    }
}
