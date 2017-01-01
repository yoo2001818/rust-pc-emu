use i8086::cpu::CPU;
use i8086::constants;

pub enum RegisterOrMemory {
    Register(u8),
    Memory(u16)
}

pub fn parse_reg(reg: u8) -> RegisterOrMemory {
    RegisterOrMemory::Register(reg)
}

pub fn parse_rm(cpu: &mut CPU) -> (RegisterOrMemory, u8) {
    let op = cpu.next_code();
    let mode = op >> 6;
    let center = (op >> 3) & 7;
    let rm = op & 7;
    if mode == 3 {
        (RegisterOrMemory::Register(rm), center)
    } else {
        let offset = match mode {
            0 if rm == 6 => cpu.next_code_word(),
            0 if rm != 6 => 0u16,
            1 => cpu.next_code() as u16,
            2 => cpu.next_code_word(),
            _ => panic!("Unknown Mode {}", mode)
        };
        let address = (match rm {
            0 => cpu.state.registers[constants::BX] + cpu.state.registers[constants::SI],
            1 => cpu.state.registers[constants::BX] + cpu.state.registers[constants::DI],
            2 => cpu.state.registers[constants::BP] + cpu.state.registers[constants::SI],
            3 => cpu.state.registers[constants::BP] + cpu.state.registers[constants::DI],
            4 => cpu.state.registers[constants::SI],
            5 => cpu.state.registers[constants::DI],
            6 if mode == 0 => 0,
            6 if mode != 0 => cpu.state.registers[constants::BP],
            7 => cpu.state.registers[constants::BX],
            _ => panic!("Unknown R/M {}", rm)
        }) + offset;
        (RegisterOrMemory::Memory(address), center)
    }
}
