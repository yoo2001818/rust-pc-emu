use std::mem;
use std::fmt;

pub enum Register {
    AX = 0,
    CX = 1,
    DX = 2,
    BX = 3,
    SP = 4,
    BP = 5,
    SI = 6,
    DI = 7
}

pub enum Segment {
    ES = 0,
    CS = 1,
    SS = 2,
    DS = 3
}

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
            self.registers[Register::AX as usize], self.registers[Register::BX as usize],
            self.registers[Register::CX as usize], self.registers[Register::DX as usize],
            self.registers[Register::SP as usize], self.registers[Register::BP as usize],
            self.registers[Register::SI as usize], self.registers[Register::DI as usize],
            self.ip,
            self.segments[Segment::ES as usize], self.segments[Segment::CS as usize],
            self.segments[Segment::SS as usize], self.segments[Segment::DS as usize]
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
