use std::mem;
use std::fmt;
use i8086::state::State;
use i8086::constants;

pub struct CPU {
    // Allocate 1MB for testing
    pub ram: [u8; 1048576],
    pub state: State
}

impl CPU {
    // As the code is fetched from the RAM itself, it's okay to fetch more than 1 byte
    // from single function
    /// Returns the address of current segment and register.
    fn get_address_register(&self, segment: usize, register: usize) -> usize {
        let register_val = self.state.registers[register];
        let segment_val = self.state.segments[segment];
        (register_val as usize) + ((segment_val as usize) << 4)
    }
    /// Returns the address of current segment and address.
    fn get_address(&self, segment: usize, address: usize) -> usize {
        let segment_val = self.state.segments[segment];
        (address as usize) + ((segment_val as usize) << 4)
    }
    /// Returns the address of instruction pointer.
    fn get_address_ip(&self) -> usize {
        let register_val = self.state.ip;
        let segment_val = self.state.segments[constants::CS];
        (register_val as usize) + ((segment_val as usize) << 4)
    }
    /// Increment IP and return current instruction
    fn next_code(&mut self) -> u8 {
        let output = self.ram[self.get_address_ip()];
        self.state.ip += 1;
        output
    }
    /// Increment IP and return current instruction word
    fn next_code_word(&mut self) -> u16 {
        let output = self.read_word_memory(self.get_address_ip());
        self.state.ip += 2;
        output
    }
    /// Wraps the address to 0xFFFFF.
    fn wrap_address(address: usize) -> usize {
        address % 0xFFFFF
    }
    /// Reads the memory as word.
    fn read_word_memory(&self, address: usize) -> u16 {
        if address >= 0xFFFFF {
            // TODO Disable this if A20 line is off?
            unsafe {
                u16::from_le(mem::transmute::<[u8; 2], u16>([
                    self.ram[CPU::wrap_address(address)],
                    self.ram[CPU::wrap_address(address + 1)]
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
    fn write_word_memory(&mut self, address: usize, value: u16) {
        let value_le = value.to_le();
        if address >= 0xFFFFF {
            // TODO Disable this if A20 line is off?
            unsafe {
                let value_arr = mem::transmute::<u16, [u8; 2]>(value_le);
                self.ram[CPU::wrap_address(address)] = value_arr[0];
                self.ram[CPU::wrap_address(address + 1)] = value_arr[1];
            }
        } else {
            // This feels so scary
            unsafe {
                let ptr = (&mut self.ram[address] as *mut u8) as *mut u16;
                *ptr = value_le;
            }
        }
    }
    fn fetch_offset(&mut self, mode: u8, rm: u8) -> usize {
        match mode {
            0 if rm == 6 => self.next_code_word() as usize,
            0 if rm != 6 => 0,
            1 => self.next_code() as usize,
            2 => self.next_code_word() as usize,
            3 => 0,
            _ => panic!("Unknown Mode {}", mode)
        }
    }
    fn get_address_opcode(&self, mode: u8, rm: u8, offset: usize) -> usize {
        let address = (match rm {
            0 => self.state.registers[constants::BX] + self.state.registers[constants::SI],
            1 => self.state.registers[constants::BX] + self.state.registers[constants::DI],
            2 => self.state.registers[constants::BP] + self.state.registers[constants::SI],
            3 => self.state.registers[constants::BP] + self.state.registers[constants::DI],
            4 => self.state.registers[constants::SI],
            5 => self.state.registers[constants::DI],
            6 if mode == 0 => 0,
            6 if mode != 0 => self.state.registers[constants::BP],
            7 => self.state.registers[constants::BX],
            _ => panic!("Unknown R/M {}", rm)
        }) as usize + offset;
        self.get_address(constants::DS, address)
    }
    /// Reads word from R/M and displacement.
    /// To use this as REG, Provide `3` in MOD.
    fn read_word(&self, mode: u8, rm: u8, offset: usize) -> u16 {
        if mode == 3 {
            // Register mode
            self.state.registers[rm as usize]
        } else {
            // Memory mode. Uhhhh...
            let address = self.get_address_opcode(mode, rm, offset);
            self.read_word_memory(address)
        }
    }
    /// Writes word from R/M and displacement.
    /// To use this as REG, Provide `3` in MOD.
    fn write_word(&mut self, mode: u8, rm: u8, offset: usize, value: u16) {
        if mode == 3 {
            // Register mode
            self.state.registers[rm as usize] = value;
        } else {
            // Memory mode. Uhhhh...
            let address = self.get_address_opcode(mode, rm, offset);
            self.write_word_memory(address, value);
        }
    }
    /// Returns reference of byte from R/M and displacement.
    /// To use this as REG, Provide `3` in MOD.
    fn get_byte(&self, mode: u8, rm: u8, offset: usize) -> &u8 {
        if mode == 3 {
            // Register mode
            self.state.get_register_byte(rm)
        } else {
            // Memory mode. Uhhhh...
            let address = self.get_address_opcode(mode, rm, offset);
            &self.ram[address]
        }
    }
    /// Returns reference of byte from R/M and displacement.
    /// To use this as REG, Provide `3` in MOD.
    fn get_byte_mut(&mut self, mode: u8, rm: u8, offset: usize) -> &mut u8 {
        if mode == 3 {
            // Register mode
            self.state.get_register_byte_mut(rm)
        } else {
            // Memory mode. Uhhhh...
            let address = self.get_address_opcode(mode, rm, offset);
            &mut self.ram[address]
        }
    }
    fn reg_to_rm_word<F>(&mut self, d: bool, operator: F) where F: Fn(&mut CPU, u16, u16) -> u16 {
        let op = self.next_code();
        let mode = op >> 6;
        let reg = (op >> 3) & 7;
        let rm = op & 7;
        let offset = self.fetch_offset(mode, rm);
        if d {
            let self_val = self.read_word(3, reg, 0);
            let other_val = self.read_word(mode, rm, offset);
            let returned = operator(self, self_val, other_val);
            self.write_word(3, reg, 0, returned);
        } else {
            let self_val = self.read_word(mode, rm, offset);
            let other_val = self.read_word(3, reg, 0);
            let returned = operator(self, self_val, other_val);
            self.write_word(mode, rm, offset, returned);
        }
    }
    fn reg_to_rm_byte<F>(&mut self, d: bool, operator: F) where F: Fn(&mut CPU, u8, u8) -> u8 {
        let op = self.next_code();
        let mode = op >> 6;
        let reg = (op >> 3) & 7;
        let rm = op & 7;
        let offset = self.fetch_offset(mode, rm);
        if d {
            let self_val = *self.get_byte(3, reg, 0);
            let other_val = *self.get_byte(mode, rm, offset);
            let returned = operator(self, self_val, other_val);
            *self.get_byte_mut(3, reg, 0) = returned;
        } else {
            let self_val = *self.get_byte(mode, rm, offset);
            let other_val = *self.get_byte(3, reg, 0);
            let returned = operator(self, self_val, other_val);
            *self.get_byte_mut(mode, rm, offset) = returned;
        }
    }
    fn reg_to_rm<B, W>(&mut self, dw: u8, byte_op: B, word_op: W)
        where B: Fn(&mut CPU, u8, u8) -> u8, W: Fn(&mut CPU, u16, u16) -> u16 {

        let d = (dw & 2) != 0;
        let w = (dw & 1) != 0;
        if w {
            self.reg_to_rm_word(d, word_op)
        } else {
            self.reg_to_rm_byte(d, byte_op)
        }
    }
    fn rm_word<F>(&mut self, operator: F) where F: Fn(&mut CPU, u16, u8) -> u16 {
        let op = self.next_code();
        let mode = op >> 6;
        let center = (op >> 3) & 7;
        let rm = op & 7;
        let offset = self.fetch_offset(mode, rm);
        let self_val = self.read_word(mode, rm, offset);
        let returned = operator(self, self_val, center);
        self.write_word(mode, rm, offset, returned);
    }
    fn rm_byte<F>(&mut self, operator: F) where F: Fn(&mut CPU, u8, u8) -> u8 {
        let op = self.next_code();
        let mode = op >> 6;
        let center = (op >> 3) & 7;
        let rm = op & 7;
        let offset = self.fetch_offset(mode, rm);
        let self_val = *self.get_byte(mode, rm, offset);
        let returned = operator(self, self_val, center);
        *self.get_byte_mut(mode, rm, offset) = returned;
    }
    fn rm<B, W>(&mut self, w: bool, byte_op: B, word_op: W)
        where B: Fn(&mut CPU, u8, u8) -> u8, W: Fn(&mut CPU, u16, u8) -> u16 {

        if w {
            self.rm_word(word_op)
        } else {
            self.rm_byte(byte_op)
        }
    }
    fn imm_to_rm_word<F>(&mut self, operator: F) where F: Fn(&mut CPU, u16, u16, u8) -> u16 {
        self.rm_word(|cpu, x, center| {
            let other_val = cpu.next_code_word();
            operator(cpu, x, other_val, center)
        })
    }
    fn imm_to_rm_byte<F>(&mut self, operator: F) where F: Fn(&mut CPU, u8, u8, u8) -> u8 {
        self.rm_byte(|cpu, x, center| {
            let other_val = cpu.next_code();
            operator(cpu, x, other_val, center)
        })
    }
    fn imm_to_rm<B, W>(&mut self, w: bool, byte_op: B, word_op: W)
        where B: Fn(&mut CPU, u8, u8, u8) -> u8, W: Fn(&mut CPU, u16, u16, u8) -> u16 {

        if w {
            self.imm_to_rm_word(word_op)
        } else {
            self.imm_to_rm_byte(byte_op)
        }
    }
    /// Executes single instruction of the CPU.
    fn execute(&mut self) {
        let mut op = self.next_code();
        match op {
            // MOV instruction
            0x88 ... 0x8B => {
                // Register / memory to / from register
                let dw = op & 3;
                self.reg_to_rm(dw, |cpu, x, y| y, |cpu, x, y| y);
            },
            0xC6 ... 0xC7 => {
                // Immediate to register / memory
                let w = (op & 1) != 0;
                self.imm_to_rm(w, |cpu, x, y, c| y, |cpu, x, y, c| y);
            },
            0xB0 ... 0xBF => {
                // Immediate to register
                let w = (op >> 3) & 1;
                let reg = op & 7;
                if w & 1 == 0 {
                    // Byte
                    let data = self.next_code();
                    *self.get_byte_mut(3, reg, 0) = data;
                } else {
                    // Word
                    let data = self.next_code_word();
                    self.write_word(3, reg, 0, data);
                }
            },
            0xA0 ... 0xA1 => {
                // Memory to accumulator
                let w = (op >> 3) & 1;
                let addr = {
                    let addr_low = self.next_code_word();
                    self.get_address(constants::DS, addr_low as usize)
                };
                if w & 1 == 0 {
                    // Byte
                    *self.state.get_register_byte_mut(constants::AL) = self.ram[addr];
                } else {
                    // Word
                    self.state.registers[constants::AX] = self.read_word_memory(addr);
                }
            },
            0xA2 ... 0xA3 => {
                // Accumulator to memory
                let w = (op >> 3) & 1;
                let addr = {
                    let addr_low = self.next_code_word();
                    self.get_address(constants::DS, addr_low as usize)
                };
                if w & 1 == 0 {
                    // Byte
                    self.ram[addr] = *self.state.get_register_byte(constants::AL);
                } else {
                    // Word
                    let value = self.state.registers[constants::AX];
                    self.write_word_memory(addr, value);
                }
            },
            0x8E | 0x8C => {
                // Register / memory to segment register
                let d = op & 2 != 0;
                op = self.next_code();
                let mode = op >> 6;
                let sr = (op >> 3) & 4;
                let rm = op & 7;
                let offset = self.fetch_offset(mode, rm);
                if d {
                    let value = self.read_word(mode, rm, offset);
                    self.state.segments[sr as usize] = value;
                } else {
                    let value = self.state.segments[sr as usize];
                    self.write_word(mode, rm, offset, value);
                }
            },
            // PUSH instruction
            0xFF => {
                // Store R/M
                self.rm_word(|cpu, val, center| {
                    assert!(center == 6);
                    cpu.state.registers[constants::SP] -= 2;
                    let address = cpu.get_address_register(constants::SS, constants::SP);
                    cpu.write_word_memory(address, val);
                    val
                });
            },
            0x50 ... 0x57 => {
                // Store register
                let val = self.state.registers[(op & 0x7) as usize];
                self.state.registers[constants::SP] -= 2;
                let address = self.get_address_register(constants::SS, constants::SP);
                self.write_word_memory(address, val);
            },
            0x06 | 0x0E | 0x16 | 0x1E => {
                // Store segment register
                let val = self.state.segments[((op >> 3) & 0x3) as usize];
                self.state.registers[constants::SP] -= 2;
                let address = self.get_address_register(constants::SS, constants::SP);
                self.write_word_memory(address, val);
            },
            // POP instruction
            0x8F => {
                // Pop R/M
                self.rm_word(|cpu, val, center| {
                    assert!(center == 0);
                    let address = cpu.get_address_register(constants::SS, constants::SP);
                    let result = cpu.read_word_memory(address);
                    cpu.state.registers[constants::SP] += 2;
                    result
                });
            },
            0x58 ... 0x5F => {
                // Pop register
                let address = self.get_address_register(constants::SS, constants::SP);
                let result = self.read_word_memory(address);
                self.state.registers[(op & 0x7) as usize] = result;
                self.state.registers[constants::SP] += 2;
            },
            0x07 | 0x0F | 0x17 | 0x1F => {
                // Pop segment register
                let address = self.get_address_register(constants::SS, constants::SP);
                let result = self.read_word_memory(address);
                self.state.segments[((op >> 3) & 0x3) as usize] = result;
                self.state.registers[constants::SP] += 2;
            },
            // Do we raise an interrupt?
            _ => panic!("Unknown instruction {:02X}", op),
        }
    }
}
