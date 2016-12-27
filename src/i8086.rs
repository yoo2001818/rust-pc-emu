struct State {
    // Main registers
    ax: u16,
    bx: u16,
    cx: u16,
    dx: u16,
    // Index registers
    si: u16,
    di: u16,
    bp: u16,
    sp: u16,
    // Program counter
    ip: u16,
    // Segment registers
    cs: u16,
    ds: u16,
    es: u16,
    ss: u16,
    // Status register
    flags: u16
}

impl State {
    // Since it is impossible to slice u16 to u8s, just convert them.
    // Also, CPU registers use big endian.
    fn get_register_byte(&self, register: u8) -> u8 {
        match register {
            0 => self.ax as u8,
            1 => self.cx as u8,
            2 => self.dx as u8,
            3 => self.bx as u8,
            4 => (self.ax >> 8) as u8,
            5 => (self.cx >> 8) as u8,
            6 => (self.dx >> 8) as u8,
            7 => (self.bx >> 8) as u8,
            _ => panic!("Unknown register identifier {}", register)
        }
    }
    fn set_register_byte(&mut self, register: u8, value: u8) {
        match register {
            0 => self.ax = (self.ax & 0xFF00) | (value as u16),
            1 => self.cx = (self.cx & 0xFF00) | (value as u16),
            2 => self.dx = (self.dx & 0xFF00) | (value as u16),
            3 => self.bx = (self.bx & 0xFF00) | (value as u16),
            4 => self.ax = (self.ax & 0xFF) | ((value as u16) << 8),
            5 => self.cx = (self.cx & 0xFF) | ((value as u16) << 8),
            6 => self.dx = (self.dx & 0xFF) | ((value as u16) << 8),
            7 => self.bx = (self.bx & 0xFF) | ((value as u16) << 8),
            _ => panic!("Unknown register identifier {}", register)
        }
    }
    fn get_register_word(&self, register: u8) -> &u16 {
        match register {
            0 => &self.ax,
            1 => &self.cx,
            2 => &self.dx,
            3 => &self.bx,
            4 => &self.sp,
            5 => &self.bp,
            6 => &self.si,
            7 => &self.di,
            _ => panic!("Unknown register identifier {}", register)
        }
    }
    fn get_register_word_mut(&mut self, register: u8) -> &mut u16 {
        match register {
            0 => &mut self.ax,
            1 => &mut self.cx,
            2 => &mut self.dx,
            3 => &mut self.bx,
            4 => &mut self.sp,
            5 => &mut self.bp,
            6 => &mut self.si,
            7 => &mut self.di,
            _ => panic!("Unknown register identifier {}", register)
        }
    }
    fn get_segment(&self, segment: u8) -> &u16 {
        match segment {
            0 => &self.es,
            1 => &self.cs,
            2 => &self.ss,
            3 => &self.ds,
            _ => panic!("Unknown segment identifier {}", segment)
        }
    }
    fn get_segment_mut(&mut self, segment: u8) -> &mut u16 {
        match segment {
            0 => &mut self.es,
            1 => &mut self.cs,
            2 => &mut self.ss,
            3 => &mut self.ds,
            _ => panic!("Unknown segment identifier {}", segment)
        }
    }
}
