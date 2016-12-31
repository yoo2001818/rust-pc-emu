use std::mem;
use std::fmt;
use i8086::constants;

pub struct State {
    pub registers: [u16; 8],
    pub segments: [u16; 4],
    // Program counter
    pub ip: u16,
    // Status register
    pub flags: u16
}

impl State {
    pub fn new() -> State {
        State {
            registers: [0; 8],
            segments: [0, 0xFFFF, 0, 0],
            ip: 0,
            flags: 0
        }
    }
    pub fn reset(&mut self) {
        self.registers = [0; 8];
        self.segments = [0, 0xFFFF, 0, 0];
        self.ip = 0;
        self.flags = 0;
    }
    pub fn get_register_byte(&self, register: u8) -> &u8 {
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
    pub fn get_register_byte_mut(&mut self, register: u8) -> &mut u8 {
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
            self.registers[constants::AX], self.registers[constants::BX],
            self.registers[constants::CX], self.registers[constants::DX],
            self.registers[constants::SP], self.registers[constants::BP],
            self.registers[constants::SI], self.registers[constants::DI],
            self.ip,
            self.segments[constants::ES], self.segments[constants::CS],
            self.segments[constants::SS], self.segments[constants::DS]
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
