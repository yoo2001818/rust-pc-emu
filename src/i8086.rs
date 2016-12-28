use std::mem;

enum Register {
    AX = 0,
    CX = 1,
    DX = 2,
    BX = 3,
    SP = 4,
    BP = 5,
    SI = 6,
    DI = 7
}

enum Segment {
    ES = 0,
    CS = 1,
    SS = 2,
    DS = 3
}

struct State {
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
        // This is pretty awkward and unsafe but it's the bestest way to
        // fetch the reference
        unsafe {
            let sliced = mem::transmute::<&mut u16, &mut [u8; 2]>(&mut self.registers[selector]);
            // little endian high 1 1 -> +1
            // big endian low 0 0 -> +1
            let offset = if high == cfg!(target_endian = "little") { 1 } else { 0 };
            &mut sliced[offset]
        }
    }
}
