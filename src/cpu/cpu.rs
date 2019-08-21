#![allow(warnings)]
use crate::mmu::MMU;

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
}

const fn op8_helper(opcode: u8) -> (usize, u64) {
   ((opcode / 16) as usize, (1 << ((opcode as u64) % 16)))
}

const fn op_block(table: [u16; 16], op8: usize) -> u16 {
   table[op8]
}

fn cb_prefix(cpu: &mut LR35902, mmu: &mut MMU, bytes: [u8;4]) {
    let cb_opcode = bytes[1];
    CB_INS_TABLE[cb_opcode as usize](cpu, mmu, bytes);
}

const fn variety_cb(opcode: u8) -> u8 {
    let (h, l) = ((opcode & 0xF0) >> 4, (opcode & 0x0F));
    let h_mod = h % 0x4;
    (h_mod * 2) + (l / 8)
}

fn half_carry_add_u8(op: u8, op2: u8) -> (bool, bool, u8) { // carry, half carry, result
    let half_carry = ((0x0F&op) + (0x0F&op2)&0x10) > 0;
    let (result, carry) = op.overflowing_add(op2);
    (carry, half_carry, op + op2)
}

/// Opcode processing and all cpu operation is actually described here, because this macro will codegen a unique function for each opcode and all the IF statements should be compiled away.
/// Not much code here yet, but we should be able all of the possible operations here and get away without severe performance penalty thanks to compile time optimisations and the final code shouldn't be too hard, though 0xCB prefix will be a headache I guess
macro_rules! g { // short for generate
    ($opcode:literal, $inst_size:literal, $timing:literal) => {
        |mut cpu, mut mmu, bytes|
        {
            let (op8, op8m) = op8_helper($opcode);
            let mut t_src: u64 = 0;
            let mut t_abs_addr: u64; // let's try to roll with this
                // I have to stop using t_src or t_dst for any addressing, I guess
            let mut t_rel_addr: i8 = 0; // is for relative jumps, why did I even treat it like this then
            let mut t_dst: u64 = 0;
            let mut enable_interrupts = false;
            let mut interrupts_enabled = true;
            let mut condition = false;

            cpu.t += $timing;
            cpu.pc += $inst_size;

            macro_rules! expand {
               ($operation:expr, $code:block) => {
                    let op_block = op_block($operation, op8);
                    if op_block & (0x1 << (16 - op8m)) > 0 $code
                }
            }

            expand!(OP_ILLEGAL, { panic!("Illegal operation {}", $opcode); });
            // Load
            expand!(OP_SRC_REGISTER_A, { t_src = cpu.a as u64; });
            expand!(OP_SRC_REGISTER_B, { t_src = cpu.b as u64; });
            expand!(OP_SRC_REGISTER_C, { t_src = cpu.c as u64; });
            expand!(OP_SRC_REGISTER_D, { t_src = cpu.d as u64; });
            expand!(OP_SRC_REGISTER_E, { t_src = cpu.e as u64; }); // source e
            expand!(OP_SRC_REGISTER_H, { t_src = cpu.h as u64; }); // source h
            expand!(OP_SRC_REGISTER_L, { t_src = cpu.l as u64; }); // source l
            expand!(OP_SRC_REGISTER_AF, { t_src = (cpu.a as u64) << 8 | (cpu.f as u64); }); // source AF
            expand!(OP_SRC_REGISTER_BC, { t_src = (cpu.b as u64) << 8 | (cpu.c as u64); }); // source BC
            expand!(OP_SRC_REGISTER_DE, { t_src = (cpu.d as u64) << 8 | (cpu.e as u64); }); // source DE
            expand!(OP_SRC_REGISTER_HL, { t_src = (cpu.h as u64) << 8 | (cpu.l as u64); }); // source HL
            expand!(OP_SRC_REGISTER_SP, { t_src = cpu.sp as u64; }); // source SP
            expand!(OP_SRC_REGISTER_PC, { t_src = cpu.pc as u64; }); // used for relative jumps etc.
            expand!(OP_ADDRESS_READ, { t_src = mmu.read_byte(t_src as usize) as u64; }); // offender
            expand!(OP_SRC_D8, { t_src = bytes[1] as u64; }); // source d8
            expand!(OP_SRC_D16, { t_src = (bytes[1] as u64) | ((bytes[2] as u64) << 8); }); // source d16
            expand!(OP_SRC_A16, { t_src = mmu.read_byte(t_src as usize) as u64; });

            expand!(OP_EXAMPLE, { t_rel_addr = t_src as i8; })
            expand!(OP_EXAMPLE, { t_abs_addr = t_src; });
            
            ///expand!(OP_SRC_I8, { t_rel_addr = bytes[1] as i8; });
            expand!(OP_SRC_STACK_OFFSET, {}); // have no idea what this is for atm, don't remember
            expand!(OP_SRC_8BIT_REL_ADDRESS, { t_src = t_src | 0xFF00; });
            expand!(OP_TRANSFORM_ADDRESS, { t_src = t_src.wrapping_add(t_rel_addr as u64); });
            expand!(OP_ADD_4_CLOCKS_CONDITION, {});
            expand!(OP_ADD_12_CLOCKS_CONDITION, {});
            // Load destination address
            expand!(OP_DST_AHLDEC, { t_dst = (cpu.h as u64) << 8 | (cpu.l as u64); let temp = t_dst - 1; cpu.h = (temp >> 8) as u8; cpu.l = temp as u8; });
            expand!(OP_DST_AHLINC, { t_dst = (cpu.h as u64) << 8 | (cpu.l as u64); let temp = t_dst + 1; cpu.h = (temp >> 8) as u8; cpu.l = temp as u8; });
            // Stack
            expand!(OP_PUSH, { cpu.push(&mut mmu, t_src as u16); });
            expand!(OP_POP, { t_src = cpu.pop(&mmu) as u64; });
            // Operations
            expand!(OP_XOR, { t_src = (cpu.a ^ (t_src as u8)) as u64; });
            expand!(OP_OR, { t_src = (cpu.a | (t_src as u8)) as u64; });
            expand!(OP_AND, { t_src = (cpu.a & (t_src as u8)) as u64; });
            expand!(OP_EXAMPLE, { 
                let (carry, half_carry, result) = half_carry_add_u8(cpu.a, t_src as u8);
                if half_carry { 
                    cpu.set_flag(Flag::H); 
                    }
                }); // todo r8 addition
            // Flags
            expand!(OP_Z_SET_ZERO, { if t_src == 0 { cpu.set_flag(Flag::Z); }});
            // Store temp
            expand!(OP_DST_REGISTER_A, { cpu.a = t_src as u8; });
            expand!(OP_DST_REGISTER_B, { cpu.b = t_src as u8; });
            expand!(OP_DST_REGISTER_C, { cpu.c = t_src as u8; });
            expand!(OP_DST_REGISTER_D, { cpu.d = t_src as u8; });
            expand!(OP_DST_REGISTER_E, { cpu.e = t_src as u8; });
            expand!(OP_DST_REGISTER_H, { cpu.h = t_src as u8; });
            expand!(OP_DST_REGISTER_L, { cpu.l = t_src as u8; });
            expand!(OP_DST_REGISTER_SP, { cpu.sp = t_src as u16; });
            expand!(OP_DST_REGISTER_HL, { cpu.h = (t_src >> 8) as u8; cpu.l = t_src as u8; });
            expand!(OP_DST_ADDRESS, { t_rel_addr = t_dst as i8; }); // this is one suspicious cast
            expand!(OP_ADDRESS_WRITE, { mmu.write_byte(t_dst as usize, t_src as u8); });

            expand!(OP_SET_N, { cpu.set_flag(Flag::N); });
            expand!(OP_SET_H, { cpu.set_flag(Flag::H); });
            expand!(OP_SET_C, { cpu.set_flag(Flag::C); });
            expand!(OP_RESET_Z, { cpu.reset_flag(Flag::Z); });
            expand!(OP_RESET_N, { cpu.reset_flag(Flag::N); });
            expand!(OP_RESET_H, { cpu.reset_flag(Flag::H); });
            expand!(OP_RESET_C, { cpu.reset_flag(Flag::C); });
            // set an appropriate flag as a condition here ?
            expand!(OP_CONDITION_Z, { condition = cpu.get_flag(Flag::Z); });
            expand!(OP_CONDITION_C, { condition = cpu.get_flag(Flag::C); });
            expand!(OP_CONDITION_NEGATE, { condition = !condition; }); // crutch for NC, NZ
            // break flow operation ?
            expand!(OP_CONDITIONAL_BREAKFLOW, { if condition { return; }} );
            // jump operations ?
            expand!(OP_CALL_STACK, { cpu.push(&mut mmu, cpu.sp + 1); });
            expand!(OP_GENERAL_JUMP, { cpu.pc = t_src as u16; }); // let's assume we have the address into t_src here
            expand!(OP_ADD_4_CLOCKS_CONDITION, { if condition { cpu.t += 4; } });
            expand!(OP_ADD_12_CLOCKS_CONDITION, { if condition { cpu.t += 12; } });

            expand!(OP_DI, { interrupts_enabled = false; }); // disable interrupts op
            expand!(OP_EI, { enable_interrupts = true; }); // enable interrupts op

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
    ($opcode:literal, $inst_size:literal, $timing:literal) => {
        |mut cpu, mut mmu, bytes|
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
                    let op_block = op_block($operation, op8);
                    if op_block & (0x1 << (16 - op8m)) > 0 $code
                }
            }
            expand!(CBOP_VARIETY, { variety = variety_cb($opcode); });
            expand!(CBOP_SRC_R_A, { t_src = cpu.a as u64;});
            expand!(CBOP_SRC_R_B, { t_src = cpu.b as u64;});
            expand!(CBOP_SRC_R_C, { t_src = cpu.c as u64;});
            expand!(CBOP_SRC_R_D, { t_src = cpu.d as u64;});
            expand!(CBOP_SRC_R_E, { t_src = cpu.e as u64;});
            expand!(CBOP_SRC_R_H, { t_src = cpu.h as u64;});
            expand!(CBOP_SRC_R_L, { t_src = cpu.l as u64;});
            expand!(CBOP_SRC_AR_HL, {});
            expand!(CBOP_BIT, {
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
const CB_OP: OpHandler = cb_prefix as OpHandler;
pub const INS_TABLE: [OpHandler; 256] = [
    g!(0x00, 1, 4),  g!(0x01, 3, 12), g!(0x02, 1, 8),  g!(0x03, 1, 8),  g!(0x04, 1, 4),  g!(0x05, 1, 4),  g!(0x06, 2, 8),  g!(0x07, 1, 4),  g!(0x08, 3, 20), g!(0x09, 1, 8),  g!(0x0A, 1, 8),  g!(0x0B, 1, 8), g!(0x0C, 1, 4),  g!(0x0D, 1, 4),  g!(0x0E, 2, 8), g!(0x0F, 1, 4),
    g!(0x10, 2, 4),  g!(0x11, 3, 12), g!(0x12, 1, 8),  g!(0x13, 1, 8),  g!(0x14, 1, 4),  g!(0x15, 1, 4),  g!(0x16, 2, 8),  g!(0x17, 1, 4),  g!(0x18, 2, 12), g!(0x19, 1, 8),  g!(0x1A, 1, 8),  g!(0x1B, 1, 8), g!(0x1C, 1, 4),  g!(0x1D, 1, 4),  g!(0x1E, 2, 8), g!(0x1F, 1, 4),
    g!(0x20, 2, 8),  g!(0x21, 3, 12), g!(0x22, 1, 8),  g!(0x23, 1, 8),  g!(0x24, 1, 4),  g!(0x25, 1, 4),  g!(0x26, 2, 8),  g!(0x27, 1, 4),  g!(0x28, 2, 8),  g!(0x29, 1, 8),  g!(0x2A, 1, 8),  g!(0x2B, 1, 8), g!(0x2C, 1, 4),  g!(0x2D, 1, 4),  g!(0x2E, 2, 8), g!(0x2F, 1, 4),
    g!(0x30, 2, 8),  g!(0x31, 3, 12), g!(0x32, 1, 8),  g!(0x33, 1, 8),  g!(0x34, 1, 12), g!(0x35, 1, 12), g!(0x36, 2, 12), g!(0x37, 1, 4),  g!(0x38, 2, 8),  g!(0x39, 1, 8),  g!(0x3A, 1, 8),  g!(0x3B, 1, 8), g!(0x3C, 1, 4),  g!(0x3D, 1, 4),  g!(0x3E, 2, 8), g!(0x3F, 1, 4),
    g!(0x40, 1, 4),  g!(0x41, 1, 4),  g!(0x42, 1, 4),  g!(0x43, 1, 4),  g!(0x44, 1, 4),  g!(0x45, 1, 4),  g!(0x46, 1, 8),  g!(0x47, 1, 4),  g!(0x48, 1, 4),  g!(0x49, 1, 4),  g!(0x4A, 1, 4),  g!(0x4B, 1, 4), g!(0x4C, 1, 4),  g!(0x4D, 1, 4),  g!(0x4E, 1, 8), g!(0x4F, 1, 4),
    g!(0x50, 1, 4),  g!(0x51, 1, 4),  g!(0x52, 1, 4),  g!(0x53, 1, 4),  g!(0x54, 1, 4),  g!(0x55, 1, 4),  g!(0x56, 1, 8),  g!(0x57, 1, 4),  g!(0x58, 1, 4),  g!(0x59, 1, 4),  g!(0x5A, 1, 4),  g!(0x5B, 1, 4), g!(0x5C, 1, 4),  g!(0x5D, 1, 4),  g!(0x5E, 1, 8), g!(0x5F, 1, 4),
    g!(0x60, 1, 4),  g!(0x61, 1, 4),  g!(0x62, 1, 4),  g!(0x63, 1, 4),  g!(0x64, 1, 4),  g!(0x65, 1, 4),  g!(0x66, 1, 8),  g!(0x67, 1, 4),  g!(0x68, 1, 4),  g!(0x69, 1, 4),  g!(0x6A, 1, 4),  g!(0x6B, 1, 4), g!(0x6C, 1, 4),  g!(0x6D, 1, 4),  g!(0x6E, 1, 8), g!(0x6F, 1, 4),
    g!(0x70, 1, 8),  g!(0x71, 1, 8),  g!(0x72, 1, 8),  g!(0x73, 1, 8),  g!(0x74, 1, 8),  g!(0x75, 1, 8),  g!(0x76, 1, 4),  g!(0x77, 1, 8),  g!(0x78, 1, 4),  g!(0x79, 1, 4),  g!(0x7A, 1, 4),  g!(0x7B, 1, 4), g!(0x7C, 1, 4),  g!(0x7D, 1, 4),  g!(0x7E, 1, 8), g!(0x7F, 1, 4),
    g!(0x80, 1, 4),  g!(0x81, 1, 4),  g!(0x82, 1, 4),  g!(0x83, 1, 4),  g!(0x84, 1, 4),  g!(0x85, 1, 4),  g!(0x86, 1, 8),  g!(0x87, 1, 4),  g!(0x88, 1, 4),  g!(0x89, 1, 4),  g!(0x8A, 1, 4),  g!(0x8B, 1, 4), g!(0x8C, 1, 4),  g!(0x8D, 1, 4),  g!(0x8E, 1, 8), g!(0x8F, 1, 4),
    g!(0x90, 1, 4),  g!(0x91, 1, 4),  g!(0x92, 1, 4),  g!(0x93, 1, 4),  g!(0x94, 1, 4),  g!(0x95, 1, 4),  g!(0x96, 1, 8),  g!(0x97, 1, 4),  g!(0x98, 1, 4),  g!(0x99, 1, 4),  g!(0x9A, 1, 4),  g!(0x9B, 1, 4), g!(0x9C, 1, 4),  g!(0x9D, 1, 4),  g!(0x9E, 1, 8), g!(0x9F, 1, 4),
    g!(0xA0, 1, 4),  g!(0xA1, 1, 4),  g!(0xA2, 1, 4),  g!(0xA3, 1, 4),  g!(0xA4, 1, 4),  g!(0xA5, 1, 4),  g!(0xA6, 1, 8),  g!(0xA7, 1, 4),  g!(0xA8, 1, 4),  g!(0xA9, 1, 4),  g!(0xAA, 1, 4),  g!(0xAB, 1, 4), g!(0xAC, 1, 4),  g!(0xAD, 1, 4),  g!(0xAE, 1, 8), g!(0xAF, 1, 4),
    g!(0xB0, 1, 4),  g!(0xB1, 1, 4),  g!(0xB2, 1, 4),  g!(0xB3, 1, 4),  g!(0xB4, 1, 4),  g!(0xB5, 1, 4),  g!(0xB6, 1, 8),  g!(0xB7, 1, 4),  g!(0xB8, 1, 4),  g!(0xB9, 1, 4),  g!(0xBA, 1, 4),  g!(0xBB, 1, 4), g!(0xBC, 1, 4),  g!(0xBD, 1, 4),  g!(0xBE, 1, 8), g!(0xBF, 1, 4),
    g!(0xC0, 1, 8),  g!(0xC1, 1, 12), g!(0xC2, 3, 12), g!(0xC3, 1, 16), g!(0xC4, 1, 12), g!(0xC5, 1, 16), g!(0xC6, 2, 8),  g!(0xC7, 1, 16), g!(0xC8, 1, 8),  g!(0xC9, 1, 16), g!(0xCA, 1, 12), CB_OP,          g!(0xCC, 3, 12), g!(0xCD, 3, 24), g!(0xCE, 2, 8), g!(0xCF, 1, 16),
    g!(0xD0, 1, 8),  g!(0xD1, 1, 12), g!(0xD2, 3, 12), g!(0xD3, 1, 0),  g!(0xD4, 1, 12), g!(0xD5, 1, 16), g!(0xD6, 2, 8),  g!(0xD7, 1, 16), g!(0xD8, 1, 8),  g!(0xD9, 1, 16), g!(0xDA, 1, 12), g!(0xDB, 1, 0), g!(0xDC, 3, 12), g!(0xDD, 1, 0),  g!(0xDE, 2, 8), g!(0xDF, 1, 16),
    g!(0xE0, 2, 12), g!(0xE1, 1, 12), g!(0xE2, 2, 8),  g!(0xE3, 1, 0),  g!(0xE4, 1, 0),  g!(0xE5, 1, 16), g!(0xE6, 2, 8),  g!(0xE7, 1, 16), g!(0xE8, 2, 16), g!(0xE9, 1, 4),  g!(0xEA, 3, 16), g!(0xEB, 1, 0), g!(0xEC, 1, 0),  g!(0xED, 1, 0),  g!(0xEE, 2, 8), g!(0xEF, 1, 16),
    g!(0xF0, 2, 12), g!(0xF1, 1, 12), g!(0xF2, 2, 8),  g!(0xF3, 1, 4),  g!(0xF4, 1, 0),  g!(0xF5, 1, 16), g!(0xF6, 2, 8),  g!(0xF7, 1, 16), g!(0xF8, 2, 12), g!(0xF9, 1, 8),  g!(0xFA, 3, 16), g!(0xFB, 1, 4), g!(0xFC, 1, 0),  g!(0xFD, 1, 0),  g!(0xFE, 2, 8), g!(0xFF, 1, 16)
];

pub const CB_INS_TABLE: [OpHandler; 256] = [
    cb_g!(0x00, 2, 8), cb_g!(0x01, 2, 8), cb_g!(0x02, 2, 8), cb_g!(0x03, 2, 8), cb_g!(0x04, 2, 8), cb_g!(0x05, 2, 8), cb_g!(0x06, 2, 16), cb_g!(0x07, 2, 8), cb_g!(0x08, 2, 8), cb_g!(0x09, 2, 8), cb_g!(0x0A, 2, 8), cb_g!(0x0B, 2, 8), cb_g!(0x0C, 2, 8), cb_g!(0x0D, 2, 8), cb_g!(0x0E, 2, 16), cb_g!(0x0F, 2, 8),
    cb_g!(0x10, 2, 8), cb_g!(0x11, 2, 8), cb_g!(0x12, 2, 8), cb_g!(0x13, 2, 8), cb_g!(0x14, 2, 8), cb_g!(0x15, 2, 8), cb_g!(0x16, 2, 16), cb_g!(0x17, 2, 8), cb_g!(0x18, 2, 8), cb_g!(0x19, 2, 8), cb_g!(0x1A, 2, 8), cb_g!(0x1B, 2, 8), cb_g!(0x1C, 2, 8), cb_g!(0x1D, 2, 8), cb_g!(0x1E, 2, 16), cb_g!(0x1F, 2, 8),
    cb_g!(0x20, 2, 8), cb_g!(0x21, 2, 8), cb_g!(0x22, 2, 8), cb_g!(0x23, 2, 8), cb_g!(0x24, 2, 8), cb_g!(0x25, 2, 8), cb_g!(0x26, 2, 16), cb_g!(0x27, 2, 8), cb_g!(0x28, 2, 8), cb_g!(0x29, 2, 8), cb_g!(0x2A, 2, 8), cb_g!(0x2B, 2, 8), cb_g!(0x2C, 2, 8), cb_g!(0x2D, 2, 8), cb_g!(0x2E, 2, 16), cb_g!(0x2F, 2, 8),
    cb_g!(0x30, 2, 8), cb_g!(0x31, 2, 8), cb_g!(0x32, 2, 8), cb_g!(0x33, 2, 8), cb_g!(0x34, 2, 8), cb_g!(0x35, 2, 8), cb_g!(0x36, 2, 16), cb_g!(0x37, 2, 8), cb_g!(0x38, 2, 8), cb_g!(0x39, 2, 8), cb_g!(0x3A, 2, 8), cb_g!(0x3B, 2, 8), cb_g!(0x3C, 2, 8), cb_g!(0x3D, 2, 8), cb_g!(0x3E, 2, 16), cb_g!(0x3F, 2, 8),
    cb_g!(0x40, 2, 8), cb_g!(0x41, 2, 8), cb_g!(0x42, 2, 8), cb_g!(0x43, 2, 8), cb_g!(0x44, 2, 8), cb_g!(0x45, 2, 8), cb_g!(0x46, 2, 16), cb_g!(0x47, 2, 8), cb_g!(0x48, 2, 8), cb_g!(0x49, 2, 8), cb_g!(0x4A, 2, 8), cb_g!(0x4B, 2, 8), cb_g!(0x4C, 2, 8), cb_g!(0x4D, 2, 8), cb_g!(0x4E, 2, 16), cb_g!(0x4F, 2, 8),
    cb_g!(0x50, 2, 8), cb_g!(0x51, 2, 8), cb_g!(0x52, 2, 8), cb_g!(0x53, 2, 8), cb_g!(0x54, 2, 8), cb_g!(0x55, 2, 8), cb_g!(0x56, 2, 16), cb_g!(0x57, 2, 8), cb_g!(0x58, 2, 8), cb_g!(0x59, 2, 8), cb_g!(0x5A, 2, 8), cb_g!(0x5B, 2, 8), cb_g!(0x5C, 2, 8), cb_g!(0x5D, 2, 8), cb_g!(0x5E, 2, 16), cb_g!(0x5F, 2, 8),
    cb_g!(0x60, 2, 8), cb_g!(0x61, 2, 8), cb_g!(0x62, 2, 8), cb_g!(0x63, 2, 8), cb_g!(0x64, 2, 8), cb_g!(0x65, 2, 8), cb_g!(0x66, 2, 16), cb_g!(0x67, 2, 8), cb_g!(0x68, 2, 8), cb_g!(0x69, 2, 8), cb_g!(0x6A, 2, 8), cb_g!(0x6B, 2, 8), cb_g!(0x6C, 2, 8), cb_g!(0x6D, 2, 8), cb_g!(0x6E, 2, 16), cb_g!(0x6F, 2, 8),
    cb_g!(0x70, 2, 8), cb_g!(0x71, 2, 8), cb_g!(0x72, 2, 8), cb_g!(0x73, 2, 8), cb_g!(0x74, 2, 8), cb_g!(0x75, 2, 8), cb_g!(0x76, 2, 16), cb_g!(0x77, 2, 8), cb_g!(0x78, 2, 8), cb_g!(0x79, 2, 8), cb_g!(0x7A, 2, 8), cb_g!(0x7B, 2, 8), cb_g!(0x7C, 2, 8), cb_g!(0x7D, 2, 8), cb_g!(0x7E, 2, 16), cb_g!(0x7F, 2, 8),
    cb_g!(0x80, 2, 8), cb_g!(0x81, 2, 8), cb_g!(0x82, 2, 8), cb_g!(0x83, 2, 8), cb_g!(0x84, 2, 8), cb_g!(0x85, 2, 8), cb_g!(0x86, 2, 16), cb_g!(0x87, 2, 8), cb_g!(0x88, 2, 8), cb_g!(0x89, 2, 8), cb_g!(0x8A, 2, 8), cb_g!(0x8B, 2, 8), cb_g!(0x8C, 2, 8), cb_g!(0x8D, 2, 8), cb_g!(0x8E, 2, 16), cb_g!(0x8F, 2, 8),
    cb_g!(0x90, 2, 8), cb_g!(0x91, 2, 8), cb_g!(0x92, 2, 8), cb_g!(0x93, 2, 8), cb_g!(0x94, 2, 8), cb_g!(0x95, 2, 8), cb_g!(0x96, 2, 16), cb_g!(0x97, 2, 8), cb_g!(0x98, 2, 8), cb_g!(0x99, 2, 8), cb_g!(0x9A, 2, 8), cb_g!(0x9B, 2, 8), cb_g!(0x9C, 2, 8), cb_g!(0x9D, 2, 8), cb_g!(0x9E, 2, 16), cb_g!(0x9F, 2, 8),
    cb_g!(0xA0, 2, 8), cb_g!(0xA1, 2, 8), cb_g!(0xA2, 2, 8), cb_g!(0xA3, 2, 8), cb_g!(0xA4, 2, 8), cb_g!(0xA5, 2, 8), cb_g!(0xA6, 2, 16), cb_g!(0xA7, 2, 8), cb_g!(0xA8, 2, 8), cb_g!(0xA9, 2, 8), cb_g!(0xAA, 2, 8), cb_g!(0xAB, 2, 8), cb_g!(0xAC, 2, 8), cb_g!(0xAD, 2, 8), cb_g!(0xAE, 2, 16), cb_g!(0xAF, 2, 8),
    cb_g!(0xB0, 2, 8), cb_g!(0xB1, 2, 8), cb_g!(0xB2, 2, 8), cb_g!(0xB3, 2, 8), cb_g!(0xB4, 2, 8), cb_g!(0xB5, 2, 8), cb_g!(0xB6, 2, 16), cb_g!(0xB7, 2, 8), cb_g!(0xB8, 2, 8), cb_g!(0xB9, 2, 8), cb_g!(0xBA, 2, 8), cb_g!(0xBB, 2, 8), cb_g!(0xBC, 2, 8), cb_g!(0xBD, 2, 8), cb_g!(0xBE, 2, 16), cb_g!(0xBF, 2, 8),
    cb_g!(0xC0, 2, 8), cb_g!(0xC1, 2, 8), cb_g!(0xC2, 2, 8), cb_g!(0xC3, 2, 8), cb_g!(0xC4, 2, 8), cb_g!(0xC5, 2, 8), cb_g!(0xC6, 2, 16), cb_g!(0xC7, 2, 8), cb_g!(0xC8, 2, 8), cb_g!(0xC9, 2, 8), cb_g!(0xCA, 2, 8), cb_g!(0xCB, 2, 8), cb_g!(0xCC, 2, 8), cb_g!(0xCD, 2, 8), cb_g!(0xCE, 2, 16), cb_g!(0xCF, 2, 8),
    cb_g!(0xD0, 2, 8), cb_g!(0xD1, 2, 8), cb_g!(0xD2, 2, 8), cb_g!(0xD3, 2, 8), cb_g!(0xD4, 2, 8), cb_g!(0xD5, 2, 8), cb_g!(0xD6, 2, 16), cb_g!(0xD7, 2, 8), cb_g!(0xD8, 2, 8), cb_g!(0xD9, 2, 8), cb_g!(0xDA, 2, 8), cb_g!(0xDB, 2, 8), cb_g!(0xDC, 2, 8), cb_g!(0xDD, 2, 8), cb_g!(0xDE, 2, 16), cb_g!(0xDF, 2, 8),
    cb_g!(0xE0, 2, 8), cb_g!(0xE1, 2, 8), cb_g!(0xE2, 2, 8), cb_g!(0xE3, 2, 8), cb_g!(0xE4, 2, 8), cb_g!(0xE5, 2, 8), cb_g!(0xE6, 2, 16), cb_g!(0xE7, 2, 8), cb_g!(0xE8, 2, 8), cb_g!(0xE9, 2, 8), cb_g!(0xEA, 2, 8), cb_g!(0xEB, 2, 8), cb_g!(0xEC, 2, 8), cb_g!(0xED, 2, 8), cb_g!(0xEE, 2, 16), cb_g!(0xEF, 2, 8),
    cb_g!(0xF0, 2, 8), cb_g!(0xF1, 2, 8), cb_g!(0xF2, 2, 8), cb_g!(0xF3, 2, 8), cb_g!(0xF4, 2, 8), cb_g!(0xF5, 2, 8), cb_g!(0xF6, 2, 16), cb_g!(0xF7, 2, 8), cb_g!(0xF8, 2, 8), cb_g!(0xF9, 2, 8), cb_g!(0xFA, 2, 8), cb_g!(0xFB, 2, 8), cb_g!(0xFC, 2, 8), cb_g!(0xFD, 2, 8), cb_g!(0xFE, 2, 16), cb_g!(0xFF, 2, 8),
];

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
        }
    }

    pub fn set_flag(&mut self, flag: Flag) {
        self.f |= 1 << (flag as u8);
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

    pub fn execute(&mut self, mmu: &mut MMU) {
        let bytes = mmu.read_ahead(self.pc as usize);
        let opcode = bytes[0];
        INS_TABLE[opcode as usize](self, mmu, bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn prerequisites() -> ( LR35902, MMU ){
        ( LR35902::new(), MMU::new() )
    }

    // not yet

    // #[test]
    // fn test_20() {
    //     let ( mut cpu, mut mmu ) = prerequisites();
    //     cpu.set_flag(Flag::Z);
    //     // pc should be 0
    //     INS_TABLE[0x20](&mut cpu, &mut mmu, [0,10,0,0]);
    //     // and probably 10 now. correction, 12, because the instruction is 2 bytes long
    //     assert_eq!(cpu.pc, 12);
    // }

    // #[test]
    // fn test_7c() { // BIT test
    //     // I'm an idiot, that's a 0xCB prefix opcode, wth was I doing
    //     let ( mut cpu, mut mmu ) = prerequisites();
    //     cpu.h = 0b0000_0000;
    //     println!("{:?}", cpu);
    //     CB_INS_TABLE[0x7C](&mut cpu, &mut mmu, [0,0,0,0]);
    //     println!("{:?}", cpu);
    //     assert_eq!(cpu.get_flag(Flag::Z), true); // same
    //     assert_eq!(cpu.get_flag(Flag::N), false);
    //     assert_eq!(cpu.get_flag(Flag::H), true); // this one is ok
    //     cpu.h = 0b1000_0000;
    //     // CB_INS_TABLE[0x7C](&mut cpu, &mut mmu, [0,0,0,0]);
    //     // assert_eq!(cpu.get_flag(Flag::Z), false); //TODO: have to check whether I need to unset the flag otherwise, probably so
    // }

    #[test]
    fn test_reset_flag() {
        let ( mut cpu, _) = prerequisites();
        assert_eq!(cpu.get_flag(Flag::N), false);
        cpu.set_flag(Flag::N);
        assert_eq!(cpu.get_flag(Flag::N), true);
        cpu.reset_flag(Flag::N);
        assert_eq!(cpu.get_flag(Flag::N), false);
        cpu.reset_flag(Flag::N);
        assert_eq!(cpu.get_flag(Flag::N), false);
    }

    #[test]
    fn test_16bitld() {
        let ( mut cpu, mut mmu ) = prerequisites();
        INS_TABLE[0x01](&mut cpu, &mut mmu, [0x00, 0xFF, 0x9F, 0x00]); // little endian, so 0x9FFF should be in BC
        assert_eq!(cpu.b, 0x9f);
        assert_eq!(cpu.c, 0xff);
        // add de, hl, sp checks
    }
}