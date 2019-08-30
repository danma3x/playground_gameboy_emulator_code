#![allow(warnings)]
use memory_bus::MMU;

use super::operations::*;

pub enum Flag { // Wrong values, too lazy to look up right now
    Z = 0x7,
    N = 0x6,
    H = 0x5,
    C = 0x4,
}

#[derive(Debug)]
pub struct LR35902 {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    pub pc: u16,
    sp: u16,
    t: usize,
    halted: bool,
}

const fn op8_helper(opcode: u8) -> (usize, u64) {
   ((opcode / 16) as usize, (1 << (15 - ((opcode as u64) % 16))))
}

const fn op_block(table: &[u16; 16], op8: usize) -> u16 {
   table[op8]
}

//fn cb_prefix(cpu: &mut LR35902, mmu: &mut MMU, bytes: [u8;4]) {
//    let cb_opcode = bytes[1];
//    CB_INS_TABLE[cb_opcode as usize](cpu, mmu, bytes);
//}

const fn variety_cb(opcode: u8) -> u8 {
    let (h, l) = ((opcode & 0xF0) >> 4, (opcode & 0x0F));
    let h_mod = h % 0x4;
    (h_mod * 2) + (l / 8)
}

fn half_carry_add_u8(op: u8, op2: u8) -> (bool, bool, u8) { // carry, half carry, result
    let half_carry = ((0x0F&op).wrapping_add(0x0F&op2)&0x10) > 0;
    let (result, carry) = op.overflowing_add(op2);
    (carry, half_carry, result)
}

fn half_carry_sub_u8(op: u8, op2: u8) -> (bool, bool, u8) { // carry, half carry, result
    let half_carry = ((0x0F&op).wrapping_sub(0x0F&op2)&0x10) > 0;
    let (result, carry) = op.overflowing_sub(op2);
    (carry, half_carry, result)
}

/// Opcode processing and all cpu operation is actually described here, because this macro will codegen a unique function for each opcode and all the IF statements should be compiled away.
/// Not much code here yet, but we should be able all of the possible operations here and get away without severe performance penalty thanks to compile time optimisations and the final code shouldn't be too hard, though 0xCB prefix will be a headache I guess
macro_rules! g { // short for generate
    ($funname:ident, $opcode:literal, $inst_size:literal, $timing:literal) => {
        fn $funname(cpu: &mut LR35902, mmu: &mut MMU, bytes: [u8;4])
        {
            let (op8, op8m) = op8_helper($opcode);
            let mut t_src: u64 = 0;
            let mut t_dst: u64 = 0;
            let mut t_result: u64 = 0;

            let mut t_abs_addr: u64 = 0;
            let mut t_rel_addr: i8 = 0;

            let mut enable_interrupts = false;
            let mut interrupts_enabled = true;
            let mut condition = false;
            let mut neg = false;
            let mut zero = false;
            let mut carry: bool = false;
            let mut half_carry: bool = false;

            cpu.t += $timing;
            cpu.pc += $inst_size;

            if $opcode == 0x76 {
                cpu.halted = true;
                return;
            }

            macro_rules! expand {
               ($operation:expr, $code:block) => {
                    let op_block = op_block(&$operation, op8);
                    if (op_block & (op8m as u16)) > 0 $code
                }
            }
            expand!(OP_ILLEGAL, { panic!("Illegal operation {}", $opcode); });
            // Load
            // immediate
            expand!(OP_SRC_D8, { t_src = bytes[1] as u64; }); // source d8
            expand!(OP_SRC_D16, { t_src = (bytes[1] as u64) | ((bytes[2] as u64) << 8); }); // source d16
            //
            expand!(OP_SRC_R_RA, { t_src = cpu.a as u64;});
            expand!(OP_SRC_R_RB, { t_src = cpu.b as u64; });
            expand!(OP_SRC_R_RC, { t_src = cpu.c as u64; });
            expand!(OP_SRC_R_RD, { t_src = cpu.d as u64; });
            expand!(OP_SRC_R_RE, { t_src = cpu.e as u64; }); // source e
            expand!(OP_SRC_R_RH, { t_src = cpu.h as u64; }); // source h
            expand!(OP_SRC_R_RL, { t_src = cpu.l as u64; }); // source l
            expand!(OP_SRC_R_RAF, { t_src = (cpu.a as u64) << 8 | (cpu.f as u64); }); // source AF
            expand!(OP_SRC_R_RBC, { t_src = (cpu.b as u64) << 8 | (cpu.c as u64); }); // source BC
            expand!(OP_SRC_R_RDE, { t_src = (cpu.d as u64) << 8 | (cpu.e as u64); }); // source DE
            expand!(OP_SRC_R_RHL, { t_src = (cpu.h as u64) << 8 | (cpu.l as u64); }); // source HL
            expand!(OP_SRC_R_RSP, { t_src = cpu.sp as u64; }); // source SP
            expand!(OP_SRC_R_RPC, { t_src = cpu.pc as u64; });
            //
            //immediate
            expand!(OP_DST_D8, { t_src = bytes[1] as u64; }); // source d8
            expand!(OP_DST_D16, { t_src = (bytes[1] as u64) | ((bytes[2] as u64) << 8); }); // source d16
            //
            expand!(OP_DST_R_RA, { t_dst = cpu.a as u64; });
            expand!(OP_DST_R_RB, { t_dst = cpu.b as u64; });
            expand!(OP_DST_R_RC, { t_dst = cpu.c as u64; });
            expand!(OP_DST_R_RD, { t_dst = cpu.d as u64; });
            expand!(OP_DST_R_RE, { t_dst = cpu.e as u64; });
            expand!(OP_DST_R_RH, { t_dst = cpu.h as u64; });
            expand!(OP_DST_R_RL, { t_dst = cpu.l as u64; });
            expand!(OP_DST_R_RBC, { t_dst = (cpu.b as u64) << 8 | (cpu.c as u64); });
            expand!(OP_DST_R_RDE, { t_dst = (cpu.d as u64) << 8 | (cpu.e as u64); });
            expand!(OP_DST_R_RHL, { t_dst = (cpu.h as u64) << 8 | (cpu.l as u64); });
            expand!(OP_DST_R_RSP, { t_dst = cpu.sp as u64; });
            expand!(OP_DST_R_RPC, { t_dst = cpu.pc as u64; });
            // OP_SRC_TO_AA
            expand!(OP_SRC_TO_AA, { t_abs_addr = t_src; });
            // OP_SRC_TO_RA
            expand!(OP_SRC_TO_RA, { t_rel_addr = t_src as i8; println!("addr {}", t_rel_addr); });
            // OP_DST_TO_AA
            expand!(OP_DST_TO_AA, { t_abs_addr = t_dst; } );
            // OP_DST_TO_RA
            expand!(OP_DST_TO_RA, { t_rel_addr = t_dst as i8; println!("addr {}", t_rel_addr); });

            // OP_COMPLEMENT
            //expand!(OP_EXAMPLE)

            expand!(OP_SRC_R_AA, { t_src = mmu.read_byte(t_abs_addr as usize) as u64; });
            expand!(OP_DST_R_AA, { t_dst = mmu.read_byte(t_abs_addr as usize) as u64; });

            // operations
            // should write to t_result generally, where we can write it however we want

            // OP_INC16
            expand!(OP_INC8, {
                let (carry, half_carry, result) = half_carry_add_u8(t_src as u8, 1);
                if result == 0 {
                    cpu.set_flag(Flag::Z);
                }
                if half_carry {
                    cpu.set_flag(Flag::H);
                }
                t_result = result as u64;
            });
            expand!(OP_XOR, { t_result = (cpu.a ^ (t_src as u8)) as u64; });
            expand!(OP_OR, { t_result = (cpu.a | (t_src as u8)) as u64; });
            expand!(OP_AND, { t_result = (cpu.a & (t_src as u8)) as u64; });
            expand!(OP_INC16, { t_result = t_src + 1; });
            expand!(OP_DEC8, {
                let (carry, half_carry, result) = half_carry_sub_u8(t_src as u8, 1);
                if result == 0 {
                    cpu.set_flag(Flag::Z);
                }
                if half_carry {
                    cpu.set_flag(Flag::H);
                }
                t_result = result as u64;
            });
            //right shift
            expand!(OP_EXAMPLE, {
                if 0x1 & t_src > 0 {
                    cpu.set_flag(Flag::C);
                }
                t_result = t_src >> 0x1;
            });
            //left shift
            expand!(OP_EXAMPLE, {
                if 0x80 & t_src > 0 {
                    cpu.set_flag(Flag::C);
                }
                t_result = t_src << 0x1;
            });
            //
            expand!(OP_EXAMPLE, {
                if cpu.get_flag(Flag::C) {
                    let (carry, half_carry, temp) = half_carry_add_u8(t_dst as u8, 1);
                    t_dst = temp as u64;
                }
            });
            expand!(OP_DEC16, { t_result = t_src - 1; });
            expand!(OP_APPLY_RA_TO_AA, { t_abs_addr = t_abs_addr.wrapping_add(t_rel_addr as u64); println!("Addr {}", t_abs_addr)});


            //POP
            expand!(OP_POP_TO_SRC, { t_src = cpu.pop(mmu) as u64; });

            expand!(OP_SRC_TO_RESULT, { t_result = t_src; });

            //PUSH
            expand!(OP_PUSH, { cpu.push(mmu, t_result as u16); });


            expand!(OP_RES_W_ADDR, { mmu.write_byte(t_abs_addr as usize, t_result as u8); });
            expand!(OP_RES_W_RA, { cpu.a = t_result as u8; });
            expand!(OP_RES_W_RB, { cpu.b = t_result as u8; });
            expand!(OP_RES_W_RC, { cpu.c = t_result as u8; });
            expand!(OP_RES_W_RD, { cpu.d = t_result as u8; });
            expand!(OP_RES_W_RE, { cpu.e = t_result as u8; });
            expand!(OP_RES_W_RH, { cpu.h = t_result as u8; });
            expand!(OP_RES_W_RL, { cpu.l = t_result as u8; });
            expand!(OP_RES_W_RAF, { cpu.a = (t_result >> 8) as u8; cpu.f = t_result as u8; });
            expand!(OP_RES_W_RBC, { cpu.b = (t_result >> 8) as u8; cpu.c = t_result as u8; });
            expand!(OP_RES_W_RDE, { cpu.d = (t_result >> 8) as u8; cpu.e = t_result as u8; });
            expand!(OP_RES_W_RHL, { cpu.h = (t_result >> 8) as u8; cpu.l = t_result as u8; });
            expand!(OP_RES_W_RSP, { cpu.sp = t_result as u16; });

            expand!(OP_HLDEC, { t_abs_addr -= 1; cpu.h = (t_abs_addr >> 8) as u8; cpu.l = t_abs_addr as u8; });
            expand!(OP_HLINC, { t_abs_addr += 1; cpu.h = (t_abs_addr >> 8) as u8; cpu.l = t_abs_addr as u8; });

            //JUMP
            expand!(OP_JUMP_CONDITION, {
                if $opcode & 0x10 > 0 { // make simpler
                    condition = cpu.get_flag(Flag::C);
                    println!("It's flag C");
                } else {
                    condition = cpu.get_flag(Flag::Z);
                    println!("It's flag Z");
                }
                if ($opcode & 0xF) < 0x8 {
                    condition = !condition;
                    println!("And N");
                }
            });
            expand!(OP_BREAK, { if !condition { return; } });
            expand!(OP_JUMP, { cpu.pc = t_abs_addr as u16; println!("{}", cpu.pc); });

            expand!(OP_DI, { interrupts_enabled = false; }); // disable interrupts op
            expand!(OP_EI, { enable_interrupts = true; }); // enable interrupts op

            expand!(OP_RESET_FLAG_Z, { cpu.reset_flag(Flag::Z); });
            expand!(OP_RESET_FLAG_N, { cpu.reset_flag(Flag::N); });
            expand!(OP_RESET_FLAG_H, { cpu.reset_flag(Flag::H); });
            expand!(OP_RESET_FLAG_C, { cpu.reset_flag(Flag::C); });
            expand!(OP_SET_FLAG_Z, { cpu.set_flag(Flag::Z); });
            expand!(OP_SET_FLAG_N, { cpu.set_flag(Flag::N); });
            expand!(OP_SET_FLAG_H, { cpu.set_flag(Flag::H); });
            expand!(OP_SET_FLAG_C, { cpu.set_flag(Flag::C); });
            expand!(OP_USE_FLAG_Z, { cpu.assign_flag(Flag::Z, zero); });
            expand!(OP_USE_FLAG_H, { cpu.assign_flag(Flag::H, half_carry); });
            expand!(OP_USE_FLAG_C, { cpu.assign_flag(Flag::C, carry); });

            if enable_interrupts {
                interrupts_enabled = true;
                enable_interrupts = false;
            }
        }
    }
}

/// Here this macro will expand in 0xFF of unique anonymous functions and we can just invoke these functions by indexing this array with the opcode
/// Inspired by Bisqwit's Nesemu1
type OpHandler = fn(cpu: &mut LR35902, mmu: &mut MMU, bytes: [u8;4]);

macro_rules! cb_g {
    ($funname:ident, $opcode:literal, $inst_size:literal, $timing:literal) => {
        fn $funname(cpu: &mut LR35902, mmu: &mut MMU, bytes: [u8;4])
        {
            let (op8, op8m) = op8_helper($opcode);
            let mut t_src: u64 = 0;
            let mut t_dst: u64 = 0;
            let mut _enable_interrupts = false; let mut _interrupts_enabled = true; // should be on cpu struct most likely, if we have two instruction tables
            let mut _condition = false;
            let mut variety=0;

            cpu.t += $timing;
            cpu.pc += $inst_size;

            macro_rules! expand {
               ($operation:expr, $code:block) => {
                    let op_block = op_block(&$operation, op8);
                    if (op_block & (op8m as u16)) > 0 $code
                }
            }
            expand!(&CBOP_VARIETY, { variety = variety_cb($opcode); });
            expand!(&CBOP_SRC_R_RA, { t_src = cpu.a as u64;});
            expand!(&CBOP_SRC_R_RB, { t_src = cpu.b as u64;});
            expand!(&CBOP_SRC_R_RC, { t_src = cpu.c as u64;});
            expand!(&CBOP_SRC_R_RD, { t_src = cpu.d as u64;});
            expand!(&CBOP_SRC_R_RE, { t_src = cpu.e as u64;});
            expand!(&CBOP_SRC_R_RH, { t_src = cpu.h as u64;});
            expand!(&CBOP_SRC_R_RL, { t_src = cpu.l as u64;});
            expand!(&CBOP_SRC_R_AHL, { t_src = mmu.read_byte(((cpu.h as usize) << 8) | cpu.l as usize) as u64; });
            expand!(&CBOP_BIT, {
                let t = (t_src & (0x1 << variety)) > 0;
                if !t {
                        cpu.set_flag(Flag::Z);
                    }
                cpu.set_flag(Flag::H);
                cpu.reset_flag(Flag::N);
            });
        }
    }
}
//const CB_OP: OpHandler = cb_prefix as OpHandler;
//pub(self) const INS_TABLE: [OpHandler; 256] = [
//    g!(0x00, 1, 4),  g!(0x01, 3, 12), g!(0x02, 1, 8),  g!(0x03, 1, 8),  g!(0x04, 1, 4),  g!(0x05, 1, 4),  g!(0x06, 2, 8),  g!(0x07, 1, 4),  g!(0x08, 3, 20), g!(0x09, 1, 8),  g!(0x0A, 1, 8),  g!(0x0B, 1, 8), g!(0x0C, 1, 4),  g!(0x0D, 1, 4),  g!(0x0E, 2, 8), g!(0x0F, 1, 4),
//    g!(0x10, 2, 4),  g!(0x11, 3, 12), g!(0x12, 1, 8),  g!(0x13, 1, 8),  g!(0x14, 1, 4),  g!(0x15, 1, 4),  g!(0x16, 2, 8),  g!(0x17, 1, 4),  g!(0x18, 2, 12), g!(0x19, 1, 8),  g!(0x1A, 1, 8),  g!(0x1B, 1, 8), g!(0x1C, 1, 4),  g!(0x1D, 1, 4),  g!(0x1E, 2, 8), g!(0x1F, 1, 4),
//    g!(0x20, 2, 8),  g!(0x21, 3, 12), g!(0x22, 1, 8),  g!(0x23, 1, 8),  g!(0x24, 1, 4),  g!(0x25, 1, 4),  g!(0x26, 2, 8),  g!(0x27, 1, 4),  g!(0x28, 2, 8),  g!(0x29, 1, 8),  g!(0x2A, 1, 8),  g!(0x2B, 1, 8), g!(0x2C, 1, 4),  g!(0x2D, 1, 4),  g!(0x2E, 2, 8), g!(0x2F, 1, 4),
//    g!(0x30, 2, 8),  g!(0x31, 3, 12), g!(0x32, 1, 8),  g!(0x33, 1, 8),  g!(0x34, 1, 12), g!(0x35, 1, 12), g!(0x36, 2, 12), g!(0x37, 1, 4),  g!(0x38, 2, 8),  g!(0x39, 1, 8),  g!(0x3A, 1, 8),  g!(0x3B, 1, 8), g!(0x3C, 1, 4),  g!(0x3D, 1, 4),  g!(0x3E, 2, 8), g!(0x3F, 1, 4),
//    g!(0x40, 1, 4),  g!(0x41, 1, 4),  g!(0x42, 1, 4),  g!(0x43, 1, 4),  g!(0x44, 1, 4),  g!(0x45, 1, 4),  g!(0x46, 1, 8),  g!(0x47, 1, 4),  g!(0x48, 1, 4),  g!(0x49, 1, 4),  g!(0x4A, 1, 4),  g!(0x4B, 1, 4), g!(0x4C, 1, 4),  g!(0x4D, 1, 4),  g!(0x4E, 1, 8), g!(0x4F, 1, 4),
//    g!(0x50, 1, 4),  g!(0x51, 1, 4),  g!(0x52, 1, 4),  g!(0x53, 1, 4),  g!(0x54, 1, 4),  g!(0x55, 1, 4),  g!(0x56, 1, 8),  g!(0x57, 1, 4),  g!(0x58, 1, 4),  g!(0x59, 1, 4),  g!(0x5A, 1, 4),  g!(0x5B, 1, 4), g!(0x5C, 1, 4),  g!(0x5D, 1, 4),  g!(0x5E, 1, 8), g!(0x5F, 1, 4),
//    g!(0x60, 1, 4),  g!(0x61, 1, 4),  g!(0x62, 1, 4),  g!(0x63, 1, 4),  g!(0x64, 1, 4),  g!(0x65, 1, 4),  g!(0x66, 1, 8),  g!(0x67, 1, 4),  g!(0x68, 1, 4),  g!(0x69, 1, 4),  g!(0x6A, 1, 4),  g!(0x6B, 1, 4), g!(0x6C, 1, 4),  g!(0x6D, 1, 4),  g!(0x6E, 1, 8), g!(0x6F, 1, 4),
//    g!(0x70, 1, 8),  g!(0x71, 1, 8),  g!(0x72, 1, 8),  g!(0x73, 1, 8),  g!(0x74, 1, 8),  g!(0x75, 1, 8),  g!(0x76, 1, 4),  g!(0x77, 1, 8),  g!(0x78, 1, 4),  g!(0x79, 1, 4),  g!(0x7A, 1, 4),  g!(0x7B, 1, 4), g!(0x7C, 1, 4),  g!(0x7D, 1, 4),  g!(0x7E, 1, 8), g!(0x7F, 1, 4),
//    g!(0x80, 1, 4),  g!(0x81, 1, 4),  g!(0x82, 1, 4),  g!(0x83, 1, 4),  g!(0x84, 1, 4),  g!(0x85, 1, 4),  g!(0x86, 1, 8),  g!(0x87, 1, 4),  g!(0x88, 1, 4),  g!(0x89, 1, 4),  g!(0x8A, 1, 4),  g!(0x8B, 1, 4), g!(0x8C, 1, 4),  g!(0x8D, 1, 4),  g!(0x8E, 1, 8), g!(0x8F, 1, 4),
//    g!(0x90, 1, 4),  g!(0x91, 1, 4),  g!(0x92, 1, 4),  g!(0x93, 1, 4),  g!(0x94, 1, 4),  g!(0x95, 1, 4),  g!(0x96, 1, 8),  g!(0x97, 1, 4),  g!(0x98, 1, 4),  g!(0x99, 1, 4),  g!(0x9A, 1, 4),  g!(0x9B, 1, 4), g!(0x9C, 1, 4),  g!(0x9D, 1, 4),  g!(0x9E, 1, 8), g!(0x9F, 1, 4),
//    g!(0xA0, 1, 4),  g!(0xA1, 1, 4),  g!(0xA2, 1, 4),  g!(0xA3, 1, 4),  g!(0xA4, 1, 4),  g!(0xA5, 1, 4),  g!(0xA6, 1, 8),  g!(0xA7, 1, 4),  g!(0xA8, 1, 4),  g!(0xA9, 1, 4),  g!(0xAA, 1, 4),  g!(0xAB, 1, 4), g!(0xAC, 1, 4),  g!(0xAD, 1, 4),  g!(0xAE, 1, 8), g!(0xAF, 1, 4),
//    g!(0xB0, 1, 4),  g!(0xB1, 1, 4),  g!(0xB2, 1, 4),  g!(0xB3, 1, 4),  g!(0xB4, 1, 4),  g!(0xB5, 1, 4),  g!(0xB6, 1, 8),  g!(0xB7, 1, 4),  g!(0xB8, 1, 4),  g!(0xB9, 1, 4),  g!(0xBA, 1, 4),  g!(0xBB, 1, 4), g!(0xBC, 1, 4),  g!(0xBD, 1, 4),  g!(0xBE, 1, 8), g!(0xBF, 1, 4),
//    g!(0xC0, 1, 8),  g!(0xC1, 1, 12), g!(0xC2, 3, 12), g!(0xC3, 1, 16), g!(0xC4, 1, 12), g!(0xC5, 1, 16), g!(0xC6, 2, 8),  g!(0xC7, 1, 16), g!(0xC8, 1, 8),  g!(0xC9, 1, 16), g!(0xCA, 1, 12), CB_OP,          g!(0xCC, 3, 12), g!(0xCD, 3, 24), g!(0xCE, 2, 8), g!(0xCF, 1, 16),
//    g!(0xD0, 1, 8),  g!(0xD1, 1, 12), g!(0xD2, 3, 12), g!(0xD3, 1, 0),  g!(0xD4, 1, 12), g!(0xD5, 1, 16), g!(0xD6, 2, 8),  g!(0xD7, 1, 16), g!(0xD8, 1, 8),  g!(0xD9, 1, 16), g!(0xDA, 1, 12), g!(0xDB, 1, 0), g!(0xDC, 3, 12), g!(0xDD, 1, 0),  g!(0xDE, 2, 8), g!(0xDF, 1, 16),
//    g!(0xE0, 2, 12), g!(0xE1, 1, 12), g!(0xE2, 2, 8),  g!(0xE3, 1, 0),  g!(0xE4, 1, 0),  g!(0xE5, 1, 16), g!(0xE6, 2, 8),  g!(0xE7, 1, 16), g!(0xE8, 2, 16), g!(0xE9, 1, 4),  g!(0xEA, 3, 16), g!(0xEB, 1, 0), g!(0xEC, 1, 0),  g!(0xED, 1, 0),  g!(0xEE, 2, 8), g!(0xEF, 1, 16),
//    g!(0xF0, 2, 12), g!(0xF1, 1, 12), g!(0xF2, 2, 8),  g!(0xF3, 1, 4),  g!(0xF4, 1, 0),  g!(0xF5, 1, 16), g!(0xF6, 2, 8),  g!(0xF7, 1, 16), g!(0xF8, 2, 12), g!(0xF9, 1, 8),  g!(0xFA, 3, 16), g!(0xFB, 1, 4), g!(0xFC, 1, 0),  g!(0xFD, 1, 0),  g!(0xFE, 2, 8), g!(0xFF, 1, 16)
//];
//
//pub(self) const CB_INS_TABLE: [OpHandler; 256] = [
//    cb_g!(0x00, 2, 8), cb_g!(0x01, 2, 8), cb_g!(0x02, 2, 8), cb_g!(0x03, 2, 8), cb_g!(0x04, 2, 8), cb_g!(0x05, 2, 8), cb_g!(0x06, 2, 16), cb_g!(0x07, 2, 8), cb_g!(0x08, 2, 8), cb_g!(0x09, 2, 8), cb_g!(0x0A, 2, 8), cb_g!(0x0B, 2, 8), cb_g!(0x0C, 2, 8), cb_g!(0x0D, 2, 8), cb_g!(0x0E, 2, 16), cb_g!(0x0F, 2, 8),
//    cb_g!(0x10, 2, 8), cb_g!(0x11, 2, 8), cb_g!(0x12, 2, 8), cb_g!(0x13, 2, 8), cb_g!(0x14, 2, 8), cb_g!(0x15, 2, 8), cb_g!(0x16, 2, 16), cb_g!(0x17, 2, 8), cb_g!(0x18, 2, 8), cb_g!(0x19, 2, 8), cb_g!(0x1A, 2, 8), cb_g!(0x1B, 2, 8), cb_g!(0x1C, 2, 8), cb_g!(0x1D, 2, 8), cb_g!(0x1E, 2, 16), cb_g!(0x1F, 2, 8),
//    cb_g!(0x20, 2, 8), cb_g!(0x21, 2, 8), cb_g!(0x22, 2, 8), cb_g!(0x23, 2, 8), cb_g!(0x24, 2, 8), cb_g!(0x25, 2, 8), cb_g!(0x26, 2, 16), cb_g!(0x27, 2, 8), cb_g!(0x28, 2, 8), cb_g!(0x29, 2, 8), cb_g!(0x2A, 2, 8), cb_g!(0x2B, 2, 8), cb_g!(0x2C, 2, 8), cb_g!(0x2D, 2, 8), cb_g!(0x2E, 2, 16), cb_g!(0x2F, 2, 8),
//    cb_g!(0x30, 2, 8), cb_g!(0x31, 2, 8), cb_g!(0x32, 2, 8), cb_g!(0x33, 2, 8), cb_g!(0x34, 2, 8), cb_g!(0x35, 2, 8), cb_g!(0x36, 2, 16), cb_g!(0x37, 2, 8), cb_g!(0x38, 2, 8), cb_g!(0x39, 2, 8), cb_g!(0x3A, 2, 8), cb_g!(0x3B, 2, 8), cb_g!(0x3C, 2, 8), cb_g!(0x3D, 2, 8), cb_g!(0x3E, 2, 16), cb_g!(0x3F, 2, 8),
//    cb_g!(0x40, 2, 8), cb_g!(0x41, 2, 8), cb_g!(0x42, 2, 8), cb_g!(0x43, 2, 8), cb_g!(0x44, 2, 8), cb_g!(0x45, 2, 8), cb_g!(0x46, 2, 16), cb_g!(0x47, 2, 8), cb_g!(0x48, 2, 8), cb_g!(0x49, 2, 8), cb_g!(0x4A, 2, 8), cb_g!(0x4B, 2, 8), cb_g!(0x4C, 2, 8), cb_g!(0x4D, 2, 8), cb_g!(0x4E, 2, 16), cb_g!(0x4F, 2, 8),
//    cb_g!(0x50, 2, 8), cb_g!(0x51, 2, 8), cb_g!(0x52, 2, 8), cb_g!(0x53, 2, 8), cb_g!(0x54, 2, 8), cb_g!(0x55, 2, 8), cb_g!(0x56, 2, 16), cb_g!(0x57, 2, 8), cb_g!(0x58, 2, 8), cb_g!(0x59, 2, 8), cb_g!(0x5A, 2, 8), cb_g!(0x5B, 2, 8), cb_g!(0x5C, 2, 8), cb_g!(0x5D, 2, 8), cb_g!(0x5E, 2, 16), cb_g!(0x5F, 2, 8),
//    cb_g!(0x60, 2, 8), cb_g!(0x61, 2, 8), cb_g!(0x62, 2, 8), cb_g!(0x63, 2, 8), cb_g!(0x64, 2, 8), cb_g!(0x65, 2, 8), cb_g!(0x66, 2, 16), cb_g!(0x67, 2, 8), cb_g!(0x68, 2, 8), cb_g!(0x69, 2, 8), cb_g!(0x6A, 2, 8), cb_g!(0x6B, 2, 8), cb_g!(0x6C, 2, 8), cb_g!(0x6D, 2, 8), cb_g!(0x6E, 2, 16), cb_g!(0x6F, 2, 8),
//    cb_g!(0x70, 2, 8), cb_g!(0x71, 2, 8), cb_g!(0x72, 2, 8), cb_g!(0x73, 2, 8), cb_g!(0x74, 2, 8), cb_g!(0x75, 2, 8), cb_g!(0x76, 2, 16), cb_g!(0x77, 2, 8), cb_g!(0x78, 2, 8), cb_g!(0x79, 2, 8), cb_g!(0x7A, 2, 8), cb_g!(0x7B, 2, 8), cb_g!(0x7C, 2, 8), cb_g!(0x7D, 2, 8), cb_g!(0x7E, 2, 16), cb_g!(0x7F, 2, 8),
//    cb_g!(0x80, 2, 8), cb_g!(0x81, 2, 8), cb_g!(0x82, 2, 8), cb_g!(0x83, 2, 8), cb_g!(0x84, 2, 8), cb_g!(0x85, 2, 8), cb_g!(0x86, 2, 16), cb_g!(0x87, 2, 8), cb_g!(0x88, 2, 8), cb_g!(0x89, 2, 8), cb_g!(0x8A, 2, 8), cb_g!(0x8B, 2, 8), cb_g!(0x8C, 2, 8), cb_g!(0x8D, 2, 8), cb_g!(0x8E, 2, 16), cb_g!(0x8F, 2, 8),
//    cb_g!(0x90, 2, 8), cb_g!(0x91, 2, 8), cb_g!(0x92, 2, 8), cb_g!(0x93, 2, 8), cb_g!(0x94, 2, 8), cb_g!(0x95, 2, 8), cb_g!(0x96, 2, 16), cb_g!(0x97, 2, 8), cb_g!(0x98, 2, 8), cb_g!(0x99, 2, 8), cb_g!(0x9A, 2, 8), cb_g!(0x9B, 2, 8), cb_g!(0x9C, 2, 8), cb_g!(0x9D, 2, 8), cb_g!(0x9E, 2, 16), cb_g!(0x9F, 2, 8),
//    cb_g!(0xA0, 2, 8), cb_g!(0xA1, 2, 8), cb_g!(0xA2, 2, 8), cb_g!(0xA3, 2, 8), cb_g!(0xA4, 2, 8), cb_g!(0xA5, 2, 8), cb_g!(0xA6, 2, 16), cb_g!(0xA7, 2, 8), cb_g!(0xA8, 2, 8), cb_g!(0xA9, 2, 8), cb_g!(0xAA, 2, 8), cb_g!(0xAB, 2, 8), cb_g!(0xAC, 2, 8), cb_g!(0xAD, 2, 8), cb_g!(0xAE, 2, 16), cb_g!(0xAF, 2, 8),
//    cb_g!(0xB0, 2, 8), cb_g!(0xB1, 2, 8), cb_g!(0xB2, 2, 8), cb_g!(0xB3, 2, 8), cb_g!(0xB4, 2, 8), cb_g!(0xB5, 2, 8), cb_g!(0xB6, 2, 16), cb_g!(0xB7, 2, 8), cb_g!(0xB8, 2, 8), cb_g!(0xB9, 2, 8), cb_g!(0xBA, 2, 8), cb_g!(0xBB, 2, 8), cb_g!(0xBC, 2, 8), cb_g!(0xBD, 2, 8), cb_g!(0xBE, 2, 16), cb_g!(0xBF, 2, 8),
//    cb_g!(0xC0, 2, 8), cb_g!(0xC1, 2, 8), cb_g!(0xC2, 2, 8), cb_g!(0xC3, 2, 8), cb_g!(0xC4, 2, 8), cb_g!(0xC5, 2, 8), cb_g!(0xC6, 2, 16), cb_g!(0xC7, 2, 8), cb_g!(0xC8, 2, 8), cb_g!(0xC9, 2, 8), cb_g!(0xCA, 2, 8), cb_g!(0xCB, 2, 8), cb_g!(0xCC, 2, 8), cb_g!(0xCD, 2, 8), cb_g!(0xCE, 2, 16), cb_g!(0xCF, 2, 8),
//    cb_g!(0xD0, 2, 8), cb_g!(0xD1, 2, 8), cb_g!(0xD2, 2, 8), cb_g!(0xD3, 2, 8), cb_g!(0xD4, 2, 8), cb_g!(0xD5, 2, 8), cb_g!(0xD6, 2, 16), cb_g!(0xD7, 2, 8), cb_g!(0xD8, 2, 8), cb_g!(0xD9, 2, 8), cb_g!(0xDA, 2, 8), cb_g!(0xDB, 2, 8), cb_g!(0xDC, 2, 8), cb_g!(0xDD, 2, 8), cb_g!(0xDE, 2, 16), cb_g!(0xDF, 2, 8),
//    cb_g!(0xE0, 2, 8), cb_g!(0xE1, 2, 8), cb_g!(0xE2, 2, 8), cb_g!(0xE3, 2, 8), cb_g!(0xE4, 2, 8), cb_g!(0xE5, 2, 8), cb_g!(0xE6, 2, 16), cb_g!(0xE7, 2, 8), cb_g!(0xE8, 2, 8), cb_g!(0xE9, 2, 8), cb_g!(0xEA, 2, 8), cb_g!(0xEB, 2, 8), cb_g!(0xEC, 2, 8), cb_g!(0xED, 2, 8), cb_g!(0xEE, 2, 16), cb_g!(0xEF, 2, 8),
//    cb_g!(0xF0, 2, 8), cb_g!(0xF1, 2, 8), cb_g!(0xF2, 2, 8), cb_g!(0xF3, 2, 8), cb_g!(0xF4, 2, 8), cb_g!(0xF5, 2, 8), cb_g!(0xF6, 2, 16), cb_g!(0xF7, 2, 8), cb_g!(0xF8, 2, 8), cb_g!(0xF9, 2, 8), cb_g!(0xFA, 2, 8), cb_g!(0xFB, 2, 8), cb_g!(0xFC, 2, 8), cb_g!(0xFD, 2, 8), cb_g!(0xFE, 2, 16), cb_g!(0xFF, 2, 8),
//];

impl LR35902 {
    pub fn new() -> LR35902 {
        LR35902 {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
            t: 0,
            halted: false,
        }
    }

    pub fn set_flag(&mut self, flag: Flag) {
        self.f |= 1 << (flag as u8);
    }

    pub fn assign_flag(&mut self, flag: Flag, state: bool) {
        let bitindex = flag as u8;
        let state = state as u8;
        self.f = self.f & !(1 << bitindex) | (state << bitindex);
    }

    pub fn reset_flag(&mut self, flag: Flag) {
        self.f &= !(1 << (flag as u8));
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        (self.f & (1 << (flag as u8))) > 0
    }

    pub fn push(&mut self, mmu: &mut MMU, value: u16) {
        mmu.write_byte((self.sp - 1) as usize, (value << 8) as u8);
        mmu.write_byte((self.sp -2) as usize, value as u8);
        self.sp -= 2;
    }

    pub fn pop(&mut self, mmu: &MMU) -> u16 {
        let l = mmu.read_byte((self.sp) as usize);
        let h = mmu.read_byte((self.sp + 1) as usize);
        self.sp += 2;
        ((h as u16) << 8) | l as u16
    }

//    pub fn execute(&mut self, mmu: &mut MMU) {
//        let bytes = mmu.read_ahead(self.pc as usize);
//        let opcode = bytes[0];
//        INS_TABLE[opcode as usize](self, mmu, bytes);
//    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn prerequisites() -> ( LR35902, MMU ){
        ( LR35902::new(), MMU::new() )
    }

    #[test]
    fn nop_test() {
        let (mut cpu, mut mmu) = prerequisites();
        g!(f0, 0x00, 1, 4); // nop
        f0(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.pc, 1);
        assert_eq!(cpu.t, 4);
    }

    #[test]
    fn ld_test() {
        let (mut cpu, mut mmu) = prerequisites();
        g!(ld_bc_d16, 0x01, 3, 12); // bc, d16
        ld_bc_d16(&mut cpu, &mut mmu, [0x0, 0x01, 0x30, 0x0]);
        assert_eq!(cpu.b, 0x30);
        assert_eq!(cpu.c, 0x01);

        g!(ld_de_d16, 0x11, 3, 12); // de, d16
        ld_de_d16(&mut cpu, &mut mmu, [0x0, 0x01, 0x30, 0x0]);
        assert_eq!(cpu.d, 0x30);
        assert_eq!(cpu.e, 0x01);

        g!(ld_hl_d16, 0x21, 3, 12); // hl, d16
        ld_hl_d16(&mut cpu, &mut mmu, [0x0, 0x01, 0x30, 0x0]);
        assert_eq!(cpu.h, 0x30);
        assert_eq!(cpu.l, 0x01);

        g!(ld_sp_d16, 0x31, 3, 12); // sp, d16
        ld_sp_d16(&mut cpu, &mut mmu, [0x0, 0x01, 0x30, 0x0]);
        assert_eq!(cpu.sp, 0x3001);

        g!(ld_abc_a, 0x02, 1, 8);
        cpu.b = 0x1; cpu.c = 0x0;
        cpu.a = 0x50;
        ld_abc_a(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(mmu.read_byte(0x100), 0x50);

        g!(ld_ade_a, 0x12, 1, 8);
        cpu.d = 0x2; cpu.e = 0x0;
        cpu.a = 0x50;
        ld_ade_a(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(mmu.read_byte(0x200), 0x50);

        g!(ld_ahlinc_a, 0x22, 1, 8);

        cpu.h = 0x3; cpu.l = 0x0;
        cpu.a = 0x50;
        ld_ahlinc_a(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(mmu.read_byte(0x300), 0x50);
        assert_eq!(cpu.h, 0x3); assert_eq!(cpu.l, 0x1);

        g!(ld_ahldec_a, 0x32, 1, 8);
        cpu.h = 0x4; cpu.l = 0x0;
        cpu.a = 0x50;
        ld_ahldec_a(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(mmu.read_byte(0x400), 0x50);
        assert_eq!(cpu.h, 0x3); assert_eq!(cpu.l, 0xFF);

        g!(ld_b_d8, 0x06, 2, 8);
        ld_b_d8(&mut cpu, &mut mmu, [0x0, 0x18, 0x0, 0x0]);
        assert_eq!(cpu.b, 0x18);

        g!(ld_d_d8, 0x16, 2, 8);
        ld_d_d8(&mut cpu, &mut mmu, [0x0, 0x18, 0x0, 0x0]);
        assert_eq!(cpu.d, 0x18);

        g!(ld_h_d8, 0x26, 2, 8);
        ld_h_d8(&mut cpu, &mut mmu, [0x0, 0x18, 0x0, 0x0]);
        assert_eq!(cpu.h, 0x18);


        g!(ld_ahl_d8, 0x36, 2, 8);
        cpu.h = 0x5;
        cpu.l = 0x0;
        ld_ahl_d8(&mut cpu, &mut mmu, [0x0, 0x18, 0x0, 0x0]);
        assert_eq!(mmu.read_byte(0x500), 0x18);

        g!(ld_a16_sp, 0x08, 3, 20);


//        let addr = 0x6000;
//        let b = 0x5B5B;
//        cpu.sp = b;
//        mmu.write_byte(addr, b);
//        ld_a16_sp(&mut cpu, &mut mmu, [0x0, 0x0, 0x60, 0x0]);
//        assert_eq!(mmu.read_byte(addr), )
    }

    #[test]
    fn inc_dec_test() {
        let (mut cpu, mut mmu) = prerequisites();
        g!(inc_bc, 0x03, 1, 8); // bc, d16
        g!(inc_de, 0x13, 1, 8);
        g!(inc_hl, 0x23, 1, 8);
        g!(inc_sp, 0x33, 1, 8);
        g!(inc_b, 0x04, 1, 4);
        g!(inc_d, 0x14, 1, 4);
        g!(inc_h, 0x24, 1, 4);
        g!(inc_ahl, 0x34, 1, 12);
        g!(dec_b, 0x05, 1, 4);
        g!(dec_d, 0x15, 1, 4);
        g!(dec_h, 0x25, 1, 4);
        g!(dec_ahl, 0x35, 1, 12);

        g!(dec_bc, 0x0B, 1, 8);
        g!(dec_de, 0x1B, 1, 8);
        g!(dec_hl, 0x2B, 1, 8);
        g!(dec_sp, 0x3B, 1, 8);
        g!(inc_c, 0x0C, 1, 4);
        g!(inc_e, 0x1C, 1, 4);
        g!(inc_l, 0x2C, 1, 4);
        g!(inc_a, 0x3C, 1, 4);
        g!(dec_c, 0x0D, 1, 4);
        g!(dec_e, 0x1D, 1, 4);
        g!(dec_l, 0x2D, 1, 4);
        g!(dec_a, 0x3D, 1, 4);

        cpu.b=0x00;
        cpu.c=0xFF;
        inc_bc(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.b, 0x1);
        assert_eq!(cpu.c, 0x0);

        cpu.d=0x00;
        cpu.e=0xFF;
        inc_de(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.d, 0x1);
        assert_eq!(cpu.e, 0x0);

        cpu.h=0x00;
        cpu.l=0xFF;
        inc_hl(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.h, 0x1);
        assert_eq!(cpu.l, 0x0);

        cpu.sp = 0x00FF;
        inc_sp(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.sp, 0x0100);

        cpu.b=0x01;
        cpu.c=0x00;
        dec_bc(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.b, 0x0);
        assert_eq!(cpu.c, 0xFF);

        cpu.d=0x01;
        cpu.e=0x00;
        dec_de(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.d, 0x0);
        assert_eq!(cpu.e, 0xFF);

        cpu.h=0x01;
        cpu.l=0x00;
        dec_hl(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.h, 0x00);
        assert_eq!(cpu.l, 0xFF);

        cpu.sp = 0x0100;
        dec_sp(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.sp, 0x00FF);

        cpu.reset_flag(Flag::H);
        cpu.b = 0x0F;
        inc_b(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.b, 0x10);
        assert_eq!(cpu.get_flag(Flag::H), true);

        cpu.reset_flag(Flag::H);
        cpu.d = 0x0F;
        inc_d(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.d, 0x10);
        assert_eq!(cpu.get_flag(Flag::H), true);

        cpu.reset_flag(Flag::H);
        cpu.h = 0x0F;
        inc_h(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.h, 0x10);
        assert_eq!(cpu.get_flag(Flag::H), true);

        cpu.reset_flag(Flag::H);
        cpu.h = 0x2;
        cpu.l = 0x0;
        inc_ahl(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(mmu.read_byte(0x200), 0x01);
        assert_eq!(cpu.get_flag(Flag::H), false);

        //dec8

        cpu.reset_flag(Flag::H);
        cpu.b = 0x10;
        dec_b(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.b, 0x0F);
        assert_eq!(cpu.get_flag(Flag::H), true);

        cpu.reset_flag(Flag::H);
        cpu.d = 0x10;
        dec_d(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.d, 0x0F);
        assert_eq!(cpu.get_flag(Flag::H), true);

        cpu.reset_flag(Flag::H);
        cpu.h = 0x10;
        dec_h(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.h, 0x0F);
        assert_eq!(cpu.get_flag(Flag::H), true);

        cpu.reset_flag(Flag::H);
        cpu.h = 0x9;
        cpu.l = 0x0;
        mmu.write_byte(0x900, 255);
        dec_ahl(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(mmu.read_byte(0x900), 0xFE);
        assert_eq!(cpu.get_flag(Flag::H), false);

        cpu.reset_flag(Flag::H);
        cpu.c = 0x0F;
        inc_c(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.c, 0x10);
        assert_eq!(cpu.get_flag(Flag::H), true);

        cpu.reset_flag(Flag::H);
        cpu.c = 0x0F;
        inc_c(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.c, 0x10);
        assert_eq!(cpu.get_flag(Flag::H), true);

        cpu.reset_flag(Flag::H);
        cpu.e = 0x0F;
        inc_e(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.e, 0x10);
        assert_eq!(cpu.get_flag(Flag::H), true);

        cpu.reset_flag(Flag::H);
        cpu.l = 0x0F;
        inc_l(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.l, 0x10);
        assert_eq!(cpu.get_flag(Flag::H), true);

        cpu.reset_flag(Flag::H);
        cpu.a = 0x0F;
        inc_a(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.a, 0x10);
        assert_eq!(cpu.get_flag(Flag::H), true);

        cpu.reset_flag(Flag::H);
        cpu.c = 0x10;
        dec_c(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.c, 0x0F);
        assert_eq!(cpu.get_flag(Flag::H), true);

        cpu.reset_flag(Flag::H);
        cpu.e = 0x10;
        dec_e(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.e, 0x0F);
        assert_eq!(cpu.get_flag(Flag::H), true);

        cpu.reset_flag(Flag::H);
        cpu.l = 0x10;
        dec_l(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.l, 0x0F);
        assert_eq!(cpu.get_flag(Flag::H), true);

        cpu.reset_flag(Flag::H);
        cpu.a = 0x10;
        dec_a(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.a, 0x0F);
        assert_eq!(cpu.get_flag(Flag::H), true);
    }

    #[test]
    fn jump_test() {
        let (mut cpu, mut mmu) = prerequisites();
        g!(jr_nz_r8, 0x20, 2, 12);
        g!(jr_nc_r8, 0x30, 2, 12);
        g!(jr_r8, 0x18, 2, 12);
        g!(jr_z_r8, 0x28, 2, 12);
        g!(jr_c_r8, 0x38, 2, 12);

        cpu.reset_flag(Flag::Z);
        cpu.pc = 0xFF;
        jr_nz_r8(&mut cpu, &mut mmu, [0x0, (0-5) as u8, 0x0, 0x0]);
        assert_eq!(cpu.pc, 0xFF - 5 + 2);

        cpu.reset_flag(Flag::C);
        cpu.pc = 0xFF;
        jr_nc_r8(&mut cpu, &mut mmu, [0x0, (0-5) as u8, 0x0, 0x0]);
        assert_eq!(cpu.pc, 0xFF - 5 + 2);

        cpu.pc = 0xFF;
        jr_r8(&mut cpu, &mut mmu, [0x0, (0-5) as u8, 0x0, 0x0]);
        assert_eq!(cpu.pc, 0xFF - 5 + 2);

        cpu.reset_flag(Flag::Z);
        cpu.pc = 0xFF;
        jr_z_r8(&mut cpu, &mut mmu, [0x0, (0-5) as u8, 0x0, 0x0]);
        assert_eq!(cpu.pc, 0x0101);
        cpu.set_flag(Flag::Z);
        jr_z_r8(&mut cpu, &mut mmu, [0x0, (0-5) as u8, 0x0, 0x0]);
        assert_eq!(cpu.pc, 0x0101 - 5 + 2);

        cpu.reset_flag(Flag::C);
        cpu.pc = 0xFF;
        jr_c_r8(&mut cpu, &mut mmu, [0x0, (0-5) as u8, 0x0, 0x0]);
        assert_eq!(cpu.pc, 0x0101);
        cpu.set_flag(Flag::C);
        jr_c_r8(&mut cpu, &mut mmu, [0x0, (0-5) as u8, 0x0, 0x0]);
        assert_eq!(cpu.pc, 0x0101 - 5 + 2);
    }

    #[test]
    fn xor_test() {
        let (mut cpu, mut mmu) = prerequisites();
        g!(xor_b, 0xA8, 1, 4);
        cpu.a = 0b1010_1010;
        cpu.b = 0b1111_1111;
        xor_b(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.a, 0b0101_0101);

        g!(xor_c, 0xA9, 1, 4);
        g!(xor_d, 0xAA, 1, 4);
        g!(xor_e, 0xAB, 1, 4);
        g!(xor_h, 0xAC, 1, 4);
        g!(xor_l, 0xAD, 1, 4);
        g!(xor_ahl, 0xAE, 1, 4);
        g!(xor_a, 0xAF, 1, 4);


    }

    #[test]
    fn and_test() {
        let (mut cpu, mut mmu) = prerequisites();
        g!(and_b, 0xA0, 1, 4);
        cpu.a = 0b1111_0000;
        cpu.b = 0b0101_0101;
        and_b(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.a, 0b0101_0000);
    }

    #[test]
    fn or_test() {
        let (mut cpu, mut mmu) = prerequisites();
        g!(or_b, 0xB0, 1, 4);
        cpu.a = 0b1010_1010;
        cpu.b = 0b0000_1111;
        or_b(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.a, 0b1010_1111);
    }

    #[test]
    fn cb_bit_test() {
        let (mut cpu, mut mmu) = prerequisites();
        cb_g!(bit_0_b, 0x40, 2, 8);
        cb_g!(bit_0_c, 0x41, 2, 8);
        cb_g!(bit_0_d, 0x42, 2, 8);
        cb_g!(bit_0_e, 0x43, 2, 8);
        cb_g!(bit_0_h, 0x44, 2, 8);
        cb_g!(bit_0_l, 0x45, 2, 8);
        cb_g!(bit_0_ahl, 0x46, 2, 8);
        cb_g!(bit_0_a, 0x47, 2, 8);

        cpu.reset_flag(Flag::Z);
        bit_0_b(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.get_flag(Flag::Z), true);
        cpu.reset_flag(Flag::Z);
        bit_0_c(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.get_flag(Flag::Z), true);
        cpu.reset_flag(Flag::Z);
        bit_0_d(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.get_flag(Flag::Z), true);
        cpu.reset_flag(Flag::Z);
        bit_0_e(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.get_flag(Flag::Z), true);
        cpu.reset_flag(Flag::Z);
        bit_0_h(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.get_flag(Flag::Z), true);
        cpu.reset_flag(Flag::Z);
        bit_0_l(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.get_flag(Flag::Z), true);
        cpu.reset_flag(Flag::Z);
        bit_0_ahl(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
        assert_eq!(cpu.get_flag(Flag::Z), true);



    }

    #[test]
    fn flag_test() {
        let (mut cpu, mut mmu) = prerequisites();
        cpu.f = 0b1000_0000;
        cpu.assign_flag(Flag::Z, true);
        assert_eq!(cpu.f, 0b1000_0000);
        cpu.assign_flag(Flag::Z, false);
        assert_eq!(cpu.f, 0);
        cpu.assign_flag(Flag::N, true);
        assert_eq!(cpu.f, 0b0100_0000);
        cpu.assign_flag(Flag::N, false);
        assert_eq!(cpu.f, 0);
    }

    fn to_bcd(val: u8) -> u8 {
        let msb = val / 10;
        let lsb = val % 10;
        msb << 4 | lsb
    }
}