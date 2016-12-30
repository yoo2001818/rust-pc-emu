use std::mem;
use std::fmt;

const AX: usize = 0;
const CX: usize = 1;
const DX: usize = 2;
const BX: usize = 3;
const SP: usize = 4;
const BP: usize = 5;
const SI: usize = 6;
const DI: usize = 7;

const AL: u8 = 0;
const CL: u8 = 1;
const DL: u8 = 2;
const BL: u8 = 3;
const AH: u8 = 4;
const CH: u8 = 5;
const DH: u8 = 6;
const BH: u8 = 7;

const ES: usize = 0;
const CS: usize = 1;
const SS: usize = 2;
const DS: usize = 3;

pub struct State {
    registers: [u16; 8],
    segments: [u16; 4],
    // Program counter
    ip: u16,
    // Status register
    flags: u16
}

impl State {
    fn new() -> State {
        State {
            registers: [0; 8],
            segments: [0, 0xFFFF, 0, 0],
            ip: 0,
            flags: 0
        }
    }
    fn reset(&mut self) {
        self.registers = [0; 8];
        self.segments = [0, 0xFFFF, 0, 0];
        self.ip = 0;
        self.flags = 0;
    }
    fn get_register_byte(&self, register: u8) -> &u8 {
        let selector = (register & 3) as usize;
        let high = (register & 4) != 0;
        // This is pretty awkward and unsafe but it's the bestest way to
        // fetch the reference
        unsafe {
            let sliced = mem::transmute::<&u16, &[u8; 2]>(&self.registers[selector]);
            // little endian high 1 1 -> +1
            // big endian low 0 0 -> +1
            let offset = if high == cfg!(target_endian = "little") { 1 } else { 0 };
            &sliced[offset]
        }
    }
    fn get_register_byte_mut(&mut self, register: u8) -> &mut u8 {
        let selector = (register & 3) as usize;
        let high = (register & 4) != 0;
        // This is pretty awkward and unsafe but it's the bestest way to fetch the reference
        unsafe {
            let sliced = mem::transmute::<&mut u16, &mut [u8; 2]>(&mut self.registers[selector]);
            // little endian high 1 1 -> +1
            // big endian low 0 0 -> +1
            let offset = if high == cfg!(target_endian = "little") { 1 } else { 0 };
            &mut sliced[offset]
        }
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ax = {:04X} bx = {:04X} cx = {:04X} dx = {:04X}\n\
            sp = {:04X} bp = {:04X} si = {:04X} di = {:04X} ip = {:04X}\n\
            es = {:04X} cs = {:04X} ss = {:04X} ds = {:04X}",
            self.registers[AX], self.registers[BX],
            self.registers[CX], self.registers[DX],
            self.registers[SP], self.registers[BP],
            self.registers[SI], self.registers[DI],
            self.ip,
            self.segments[ES], self.segments[CS],
            self.segments[SS], self.segments[DS]
        )
    }
}

#[test]
fn register_byte_test() {
    let mut state = State::new();
    state.registers = [0x501, 0x602, 0x703, 0x804, 0, 0, 0, 0];
    assert!(*(state.get_register_byte(0)) == 1);
    assert!(*(state.get_register_byte(4)) == 5);
    assert!(*(state.get_register_byte(1)) == 2);
    assert!(*(state.get_register_byte(5)) == 6);
    *(state.get_register_byte_mut(0)) = 0x22;
    assert!(*(state.get_register_byte(0)) == 0x22);
    println!("{}", state);
}

pub struct CPU {
    // Allocate 1MB for testing
    ram: [u8; 1048576],
    state: State
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
        let segment_val = self.state.segments[CS];
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
            0 => self.state.registers[BX] + self.state.registers[SI],
            1 => self.state.registers[BX] + self.state.registers[DI],
            2 => self.state.registers[BP] + self.state.registers[SI],
            3 => self.state.registers[BP] + self.state.registers[DI],
            4 => self.state.registers[SI],
            5 => self.state.registers[DI],
            6 if mode == 0 => 0,
            6 if mode != 0 => self.state.registers[BP],
            7 => self.state.registers[BX],
            _ => panic!("Unknown R/M {}", rm)
        }) as usize + offset;
        self.get_address(DS, address)
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
    fn imm_to_rm_word<F>(&mut self, operator: F) where F: Fn(&mut CPU, u16, u16, u8) -> u16 {
        let op = self.next_code();
        let mode = op >> 6;
        let center = (op >> 3) & 7;
        let rm = op & 7;
        let offset = self.fetch_offset(mode, rm);
        let self_val = self.read_word(mode, rm, offset);
        let other_val = self.next_code_word();
        let returned = operator(self, self_val, other_val, center);
        self.write_word(mode, rm, offset, returned);
    }
    fn imm_to_rm_byte<F>(&mut self, operator: F) where F: Fn(&mut CPU, u8, u8, u8) -> u8 {
        let op = self.next_code();
        let mode = op >> 6;
        let center = (op >> 3) & 7;
        let rm = op & 7;
        let offset = self.fetch_offset(mode, rm);
        let self_val = *self.get_byte(mode, rm, offset);
        let other_val = self.next_code();
        let returned = operator(self, self_val, other_val, center);
        *self.get_byte_mut(mode, rm, offset) = returned;
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
                    self.get_address(DS, addr_low as usize)
                };
                if w & 1 == 0 {
                    // Byte
                    *self.state.get_register_byte_mut(AL) = self.ram[addr];
                } else {
                    // Word
                    self.state.registers[AX] = self.read_word_memory(addr);
                }
            },
            0xA2 ... 0xA3 => {
                // Accumulator to memory
                let w = (op >> 3) & 1;
                let addr = {
                    let addr_low = self.next_code_word();
                    self.get_address(DS, addr_low as usize)
                };
                if w & 1 == 0 {
                    // Byte
                    self.ram[addr] = *self.state.get_register_byte(AL);
                } else {
                    // Word
                    let value = self.state.registers[AX];
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
            // Do we raise an interrupt?
            _ => panic!("Unknown instruction {:02X}", op),
        }
    }
}
