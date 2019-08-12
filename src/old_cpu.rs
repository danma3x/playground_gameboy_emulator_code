use crate::mmu::MMU;

#[derive(Debug, Clone, Copy)]
pub enum SingleRegister {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
}

#[derive(Debug, Clone, Copy)]
pub enum DoubleRegister {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

#[derive(Debug, Clone, Copy)]
pub enum Flag {
    C = 0x0,
    N = 0x1,
    UNUSED_1 = 0x2,
    UNUSED_2 = 0x3,
    H = 0x4,
    UNUSED_3 = 0x5,
    Z = 0x6,
    S = 0x7,
}

#[derive(Debug)]
pub enum Instruction {
    Nop,
    LdR16D16(DoubleRegister, u16),
    LdR8D8(SingleRegister, u8),
    LdACA,
    LdAAC,
    LdR8R8(SingleRegister, SingleRegister),
    XorR8(SingleRegister),
    IncR8(SingleRegister),
    IncR16(DoubleRegister),
    LdHA8A(u8),
    LdHAA8(u8),
    LdAHLR8(SingleRegister),
    LdBCD16(u8, u8),
    LdSPD16(u16),
    LdHLD16(u8, u8),
    LdHLDecA,
    CallA16(u16),
    Ret,
    XorA,
    Bit7H,
    JRNZ(i8),
}

#[derive(Default, Debug)]
pub struct Z80 {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    pub pc: u16,
    pub sp: u16,
    pub t: usize,
}

/// LE transform
const fn u8_pair_to_u16_le(op_1: u8, op_2: u8) -> u16 {
    ((op_2 as u16) << 8) | (op_1 as u16)
}

/// LE transform
const fn u16_to_u8_pair_le(op: u16) -> (u8, u8) {
    ((op) as u8, (op >> 8) as u8)
}

const fn half_carry(a: u8, b: u8) -> bool {
    ((0b00001111&a) + (0b00001111&0xb)&0b0001_0000) > 0
}

impl Z80 {
    fn time(&mut self, t: usize) {
        self.t += t;
    }
    fn pc(&mut self, pc: u16) { self.pc += pc; }

    fn ldbc_d16(&mut self, h: u8, l: u8) {
        self.b = h;
        self.c = l;

        self.pc += 3;
    }

    fn ldhl_d16(&mut self, h: u8, l: u8) {
        self.h = h;
        self.l = l;

        self.pc += 3;
    }

    fn ldsp_d16(&mut self, data: u16) {
        self.sp = data;

        self.pc += 3;
    }

    fn xor_a(&mut self) {
        let reg = self.get_single_reg(SingleRegister::A);
        self.a ^= reg;
        if self.a == 0 {
            self.set_flag(Flag::Z);
        }
        self.reset_flag(Flag::N);
        self.reset_flag(Flag::H);
        self.reset_flag(Flag::C);

        self.pc += 1;
    }

    fn jr_nz(&mut self, rel: i8) {
        if !self.get_flag(Flag::Z) {
            self.pc = (self.pc as i64 + rel as i64) as u16;
        }
    }

    fn cb_bit_7_h(&mut self) {
        let res = (self.h & (0x1 >> 7));
        if res != 0 {
            self.set_flag(Flag::Z);
        }

        self.pc += 2;
    }

    fn ldhldec_a(&mut self, mmu: &mut MMU) {
        let mut reg = self.get_double_reg(DoubleRegister::HL);
        let a = self.get_single_reg(SingleRegister::A);
        mmu.write_byte(reg as usize, a);
        reg -= 1;
        self.set_double_reg(DoubleRegister::HL, reg);

        self.pc += 1;
    }

    fn ld_ac_a(&mut self, mmu: &mut MMU ) {
        let reg_a = self.get_single_reg(SingleRegister::A);
        let reg_c = self.get_single_reg(SingleRegister::C);
        let address:  u16 = 0xFF00 | (reg_c as u16);
        mmu.write_byte(address as usize, reg_a);
        self.pc += 1;
    }

    fn ld_a_ac(&mut self, mmu: &mut MMU ) {
        let reg_a = self.get_single_reg(SingleRegister::A);
        let reg_c = self.get_single_reg(SingleRegister::C);
        let address:  u16 = 0xFF00 | (reg_c as u16);
        let byte= mmu.read_byte(address as usize);
        self.set_single_reg(SingleRegister::A, byte);
        self.pc += 1;
    }

    fn ldha8a(&mut self, mmu: &mut MMU, address: u8) {
        let a = self.get_single_reg(SingleRegister::A);
        let address = (0xFF00 as usize) | (address as usize);
        mmu.write_byte(address, a);
        self.pc += 2;
    }

    fn ldhaa8(&mut self, mmu: &MMU, address: u8) {
        let address = (0xFF00 as usize) | (address as usize);
        let byte = mmu.read_byte(address as usize);
        self.set_single_reg(SingleRegister::A, byte);
        self.pc += 2;
    }

    fn get_single_reg(&self, reg: SingleRegister) -> u8 {
        match reg {
            SingleRegister::A => self.a,
            SingleRegister::B => self.b,
            SingleRegister::C => self.c,
            SingleRegister::D => self.d,
            SingleRegister::E => self.e,
            SingleRegister::F => self.f,
            SingleRegister::H => self.h,
            SingleRegister::L => self.l,
        }
    }

    fn get_double_reg(&self, reg: DoubleRegister) -> u16 {
        match reg {
            DoubleRegister::AF => u8_pair_to_u16_le(self.f, self.a),
            DoubleRegister::BC => u8_pair_to_u16_le(self.c, self.b),
            DoubleRegister::DE => u8_pair_to_u16_le(self.e, self.d),
            DoubleRegister::HL => u8_pair_to_u16_le(self.l, self.h),
            DoubleRegister::PC => self.pc,
            DoubleRegister::SP => self.sp,
        }
    }

    fn set_single_reg(&mut self, reg: SingleRegister, v: u8) {
        match reg {
            SingleRegister::A => self.a = v,
            SingleRegister::B => self.b = v,
            SingleRegister::C => self.c = v,
            SingleRegister::D => self.d = v,
            SingleRegister::E => self.e = v,
            SingleRegister::F => self.f = v,
            SingleRegister::H => self.h = v,
            SingleRegister::L => self.l = v,
        }
    }

    fn cp_r8(&mut self) {

    }

    fn push(&mut self, mmu: &mut MMU, data: u8) {
        self.sp -= 1;
        mmu.write_byte(self.sp as usize, data);
    }

    fn pop(&mut self, mmu: &mut MMU) -> u8 {
        let ret = mmu.read_byte(self.sp as usize);
        self.sp += 1;
        ret
    }

    fn call_a16(&mut self, mmu: &mut MMU, address: u16) {
        let (l, h) = u16_to_u8_pair_le(self.pc);
        self.push(mmu, h);
        self.push(mmu, l);
        self.pc = address;
    }

    fn ret(&mut self, mmu: &mut MMU) {
        let l = self.pop(mmu);
        let h = self.pop(mmu);
        self.pc = u8_pair_to_u16_le(l, h);
    }

    fn inc_r8(&mut self, reg: SingleRegister) {
        let mut val = self.get_single_reg(reg);
        let hc = half_carry(val, 1);
        if hc {
            self.set_flag(Flag::H);
        }
        val += 1;
        if val == 0 {
            self.set_flag(Flag::Z);
        }
        self.reset_flag(Flag::N);
        self.set_single_reg(reg, val);
        self.pc += 1;
    }

    fn dec_r8(&mut self, reg: SingleRegister) {
        let mut reg = self.get_single_reg(reg);
        self.set_flag(Flag::N);
    }

    fn set_double_reg(&mut self, reg: DoubleRegister, v: u16) {
        match reg {
            DoubleRegister::AF => {
                let pair = u16_to_u8_pair_le(v);
                self.f = pair.0; self.a = pair.1;
            },
            DoubleRegister::BC => {
                let pair = u16_to_u8_pair_le(v);
                self.c = pair.0; self.b = pair.1;
            },
            DoubleRegister::DE => {
                let pair = u16_to_u8_pair_le(v);
                self.e = pair.0; self.d = pair.1;
            },
            DoubleRegister::HL => {
                let pair = u16_to_u8_pair_le(v);
                self.l = pair.0; self.h = pair.1;
            },
            DoubleRegister::PC => self.pc = v,
            DoubleRegister::SP => self.sp = v,
        }
    }

    fn set_flag(&mut self, flag: Flag) {
        self.f |= 1 << (flag as u8);
    }

    const fn get_flag(&self, flag: Flag) -> bool {
        (self.f & (1<<(flag as u8))) > 0
    }

    fn reset_flag(&mut self, flag: Flag) {
        let mask: u8 = 0xFF ^ (0x1 << (flag as u8));
        self.f &= mask;
    }

    fn xor_r8(&mut self, reg: SingleRegister) {
        let reg = self.get_single_reg(reg);
        self.a ^= reg;
        if self.a == 0 {
            self.set_flag(Flag::Z);
        }
        self.reset_flag(Flag::N);
        self.reset_flag(Flag::H);
        self.reset_flag(Flag::C);
    }

    pub fn decode_next(&mut self, mmu :&mut MMU) -> Instruction {
        let bytes = mmu.read_ahead(self.pc as usize);
        let opcode = bytes[0];
        match opcode {
            0x00 => Instruction::Nop,
            0x01 => Instruction::LdR16D16(DoubleRegister::BC, u8_pair_to_u16_le(bytes[1], bytes[2])),
            0x11 => Instruction::LdR16D16(DoubleRegister::DE, u8_pair_to_u16_le(bytes[1], bytes[2])),
            0x21 => Instruction::LdR16D16(DoubleRegister::HL, u8_pair_to_u16_le(bytes[1], bytes[2])),
            0x31 => Instruction::LdR16D16(DoubleRegister::SP, u8_pair_to_u16_le(bytes[1], bytes[2])),
            0xE2 => Instruction::LdACA,
            0xF2 => Instruction::LdAAC,
            0x06 => Instruction::LdR8D8(SingleRegister::B, bytes[1]),
            0x16 => Instruction::LdR8D8(SingleRegister::D, bytes[1]),
            0x26 => Instruction::LdR8D8(SingleRegister::H, bytes[1]),
            0x0E => Instruction::LdR8D8(SingleRegister::C, bytes[1]),
            0x1E => Instruction::LdR8D8(SingleRegister::E, bytes[1]),
            0x2E => Instruction::LdR8D8(SingleRegister::L, bytes[1]),
            0x3E => Instruction::LdR8D8(SingleRegister::A, bytes[1]),
            0x0C => Instruction::IncR8(SingleRegister::C),
            0x1C => Instruction::IncR8(SingleRegister::E),
            0x2C => Instruction::IncR8(SingleRegister::L),
            0x3C => Instruction::IncR8(SingleRegister::A),

            0x04 => Instruction::IncR8(SingleRegister::B),
            0x14 => Instruction::IncR8(SingleRegister::D),
            0x24 => Instruction::IncR8(SingleRegister::H),

            0xE0 => Instruction::LdHA8A(bytes[1]),
            0xF0 => Instruction::LdHAA8(bytes[1]),

            0xCD => Instruction::CallA16(u8_pair_to_u16_le(bytes[1], bytes[2])),
            0xC9 => Instruction::Ret,

            0x70 => Instruction::LdAHLR8(SingleRegister::B),
            0x71 => Instruction::LdAHLR8(SingleRegister::C),
            0x72 => Instruction::LdAHLR8(SingleRegister::D),
            0x73 => Instruction::LdAHLR8(SingleRegister::E),
            0x74 => Instruction::LdAHLR8(SingleRegister::H),
            0x75 => Instruction::LdAHLR8(SingleRegister::L),
            0x76 => Instruction::Nop, // HALT
            0x77 => Instruction::LdAHLR8(SingleRegister::A),

            0xA8 => Instruction::XorR8(SingleRegister::B),
            0xA9 => Instruction::XorR8(SingleRegister::C),
            0xAA => Instruction::XorR8(SingleRegister::D),
            0xAB => Instruction::XorR8(SingleRegister::E),
            0xAC => Instruction::XorR8(SingleRegister::H),
            0xAD => Instruction::XorR8(SingleRegister::L),
            0xAF => Instruction::XorR8(SingleRegister::A),
            0x32 => Instruction::LdHLDecA,
            0x20 => Instruction::JRNZ(bytes[1] as i8),
//            0x01 => Instruction::Ld(Operand::Primitive(Primitive::Pair(bytes[1], bytes[2])), Operand::DoubleRegister(DoubleRegister::BC)),
//            0x31 => Instruction::Ld(Operand::Primitive(Primitive::Pair(bytes[1], bytes[2])), Operand::DoubleRegisterX(DoubleRegisterX::SP)),
//            0x32 => Instruction::Ld(Operand::SingleRegister(SingleRegister::A), Operand::Address(Value::SingleRegister(SingleRegister::A), Suffix::Decrement)),
//            0xAF => Instruction::Xor(Operand::SingleRegister(SingleRegister::A)),
//            0x21 => Instruction::Ld(Operand::Primitive(Primitive::Pair(bytes[1], bytes[2])), Operand::DoubleRegister(DoubleRegister::HL)),
//            0b0011_0010 => Instruction::Nop, // LD (HL-), HL decrement, A to address
            0xCB => {
                let cb_bytes = &bytes[1..3];
                let cb_opcode = cb_bytes[0];
                match cb_opcode {
                    0x7C => Instruction::Bit7H, // BIT 7, H
                    _ => Instruction::Nop
                }
            }
            _ => Instruction::Nop
        }
    }

    pub fn step(&mut self, mmu: &mut MMU){
        let i = self.decode_next(mmu);
        self.execute(mmu, i);
    }

    pub fn execute(&mut self, mmu: &mut MMU, instruction: Instruction) {
        println!("instruction {:?}", instruction);
        match instruction {
            Instruction::Nop => {
                self.time(4);
                self.pc += 1;
            }
            Instruction::LdR16D16(reg, value) => {
                self.set_double_reg(reg, value);
                self.pc += 3;
            }
            Instruction::LdR8D8(reg, value) => {
                self.set_single_reg(reg, value);
                self.pc += 2;
            }
            Instruction::LdACA => {
                self.ld_ac_a(mmu);
            }
            Instruction::LdAAC => {
                self.ld_a_ac(mmu);
            }
            Instruction::LdHAA8(address) => {
                self.ldhaa8(mmu, address);
            }
            Instruction::LdHA8A(address) => {
                self.ldha8a(mmu, address);
            }
            Instruction::CallA16(address) => {
                self.call_a16(mmu, address);
            }
            Instruction::Ret => {
                self.ret(mmu);
            }
            Instruction::IncR8(reg) => {
                self.inc_r8(reg);
            }
            Instruction::XorR8(reg) => {
                self.xor_r8(reg);
                self.pc += 1;
            }
            Instruction::XorA => {
                self.xor_a();
            }
            Instruction::LdHLDecA => {
                self.ldhldec_a(mmu);
            }
            Instruction::Bit7H => {
                self.cb_bit_7_h();
            }
            Instruction::JRNZ(rel) => {
                self.jr_nz(rel);
                self.pc += 2;
            }
            _ => ()
        }
    }
}
#[cfg(test)]
mod  tests {
    use super::*;
    use core::num::FpCategory::Infinite;


    fn assets() -> (Z80, MMU) {
        (Z80::default(), MMU::new())
    }

    #[test]
    fn test_bit_7() {
        let (mut cpu, mut mmu) = assets();
        //TODO: instruction test
        cpu.h = 0b1000_0000;
        cpu.cb_bit_7_h();
        assert_eq!(cpu.f, 0);
    }

    #[test]
    fn test_xor() {
        let (mut cpu, mut mmu) = assets();
        cpu.execute(&mut mmu, Instruction::XorR8(SingleRegister::A));
        assert_eq!(cpu.f, 64);
    }

    #[test]
    fn test_ldd16() {
        let (mut cpu, mut mmu) = assets();
        cpu.execute(&mut mmu, Instruction::LdR16D16(DoubleRegister::HL, 0xFFFE));
        let res = u8_pair_to_u16_le(cpu.l, cpu.h);
        assert_eq!(cpu.h, 0xFF);
        assert_eq!(cpu.l, 0xFE);
        assert_eq!(res, 0x1516);
    }

    #[test]
    fn test_me() {
        let (mut cpu, mut mmu) = assets();
        cpu.execute(&mut mmu, Instruction::LdR16D16(DoubleRegister::BC, 0x1015));
        let res = u8_pair_to_u16_le(cpu.c, cpu.b);
        assert_eq!(cpu.c, 0x15);
        assert_eq!(cpu.b, 0x10);
        assert_eq!(res, 0x1015);
    }

    #[test]
    fn test_conversion() {
        let byte = u8_pair_to_u16_le(0x15, 0x16);
        let (l, h) = u16_to_u8_pair_le(0x1516);
        assert_eq!(byte, 0x1615);
        assert_eq!(h, 0x15);
        assert_eq!(l, 0x16);
    }
}