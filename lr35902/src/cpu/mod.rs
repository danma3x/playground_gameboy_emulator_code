#![allow(warnings)]
use memory_bus::MMU;


use super::operations::*;

#[derive(Debug, Clone, Copy)]
pub enum Flag {
    Z = 0x7,
    N = 0x6,
    H = 0x5,
    C = 0x4,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Clocks {
    pub total: usize,
    pub current: u8,
}

impl Clocks {
    pub fn add(&mut self, t: u8) {
        self.total += t as usize;
        self.current += t;
    }
}

#[derive(Clone, Copy)]
struct PendingOperation {
    timing: TimingHandler,
    handler: OpHandler,
}

impl Default for PendingOperation {
    fn default() -> PendingOperation {
        PendingOperation {
            timing: |_cpu: &LR35902| { 0 },
            handler: |_cpu: &mut LR35902, _mmu: &mut MMU, _bytes: [u8; 4]| {},
        }
    }
}

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
    halted: bool,
    pub clocks: Clocks,
    bytes: [u8; 4],
    opcode: u8,
    next_operation: PendingOperation,
    pending_clocks: u8,
}

impl LR35902 {
    pub fn new() -> LR35902 {
        let clocks = Default::default();
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
            clocks,
            halted: false,
            bytes: [0;4],
            opcode: 0,
            pending_clocks: 0,
            next_operation: PendingOperation::default(),
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
        self.sp -= 2;
        mmu.write_byte((self.sp) as usize, value as u8);
        mmu.write_byte((self.sp + 1) as usize, (value >> 8) as u8);
        
    }

    pub fn pop(&mut self, mmu: &MMU) -> u16 {
        let l = mmu.read_byte((self.sp) as usize);
        let h = mmu.read_byte((self.sp + 1) as usize);
        self.sp += 2;
        ((h as u16) << 8) | l as u16
    }

    #[cfg(feature = "instruction_table")]
    pub fn init(&mut self, mmu: &mut MMU) {
        use instructions::INS_TABLE;
        self.bytes = mmu.read_ahead(self.pc as usize);
        self.opcode = self.bytes[0];
        self.next_operation = INS_TABLE[self.opcode as usize];
    }

    #[cfg(feature = "instruction_table")]
    pub fn step(&mut self, mmu: &mut MMU, clocks: u8) {
        use instructions::{ INS_TABLE, CB_INS_TABLE };
        
        self.clocks.add(clocks);
        if self.clocks.current >= self.pending_clocks {
            self.clocks.current -= self.pending_clocks;
            self.execute(mmu, self.next_operation.handler);
            self.bytes = mmu.read_ahead(self.pc as usize);
            self.opcode = self.bytes[0];
            if self.opcode == 0xCB {
                self.next_operation = CB_INS_TABLE[self.bytes[1] as usize];
                self.pending_clocks = (self.next_operation.timing)(&self);
            } else {
                self.next_operation = INS_TABLE[self.opcode as usize];
                self.pending_clocks = (self.next_operation.timing)(&self);
            }
            
        }
    }

    pub fn execute(&mut self, mmu: &mut MMU, handler: OpHandler) {
        (handler)(self, mmu, self.bytes);
    }
}

const fn op8_helper(opcode: u8) -> (usize, u64) {
   ((opcode / 16) as usize, (1 << (15 - ((opcode as u64) % 16))))
}

const fn op_block(table: &[u16; 16], op8: usize) -> u16 {
   table[op8]
}

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
        pub(super) const $funname: PendingOperation = PendingOperation{
            timing: 
                |cpu: &LR35902| -> u8
            {
                let (op8, op8m) = op8_helper($opcode);
                let timing = $timing;
                macro_rules! expand {
                ($operation:expr, $code:block) => {
                        let op_block = op_block(&$operation, op8);
                        if (op_block & (op8m as u16)) > 0 $code
                    }
                }
                timing
            },
            handler: 
                |cpu: &mut LR35902, mmu: &mut MMU, bytes: [u8;4]|
            {

                let (op8, op8m) = op8_helper($opcode);
                let mut t_src: u16 = 0;
                let mut t_dst: u16 = 0;
                let mut t_result: u16 = 0;

                let mut t_abs_addr: u16 = 0;
                let mut t_rel_addr: i8 = 0;

                let mut enable_interrupts = false;
                let mut interrupts_enabled = true;
                let mut condition = false;
                let mut neg = false;
                let mut zero = false;
                let mut carry: bool = false;
                let mut half_carry: bool = false;

                // cpu.clocks.add($timing);
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
                
                expand!(OP_JUMP_CONDITION, {
                    if $opcode & 0x10 > 0 { // make simpler
                        condition = cpu.get_flag(Flag::C);
                    } else {
                        condition = cpu.get_flag(Flag::Z);
                    }
                    if ($opcode & 0xF) < 0x8 {
                        condition = !condition;
                    }
                });
                expand!(OP_BREAK, { if !condition { return; } });
                // Load
                // immediate
                expand!(OP_SRC_D8, { t_src = bytes[1] as u16; }); // source d8
                expand!(OP_SRC_D16, { t_src = (bytes[1] as u16) | ((bytes[2] as u16) << 8); }); // source d16
                //
                expand!(OP_SRC_R_RA, { t_src = cpu.a as u16;});
                expand!(OP_SRC_R_RB, { t_src = cpu.b as u16; });
                expand!(OP_SRC_R_RC, { t_src = cpu.c as u16; });
                expand!(OP_SRC_R_RD, { t_src = cpu.d as u16; });
                expand!(OP_SRC_R_RE, { t_src = cpu.e as u16; }); // source e
                expand!(OP_SRC_R_RH, { t_src = cpu.h as u16; }); // source h
                expand!(OP_SRC_R_RL, { t_src = cpu.l as u16; }); // source l
                expand!(OP_SRC_R_RAF, { t_src = (cpu.a as u16) << 8 | (cpu.f as u16); }); // source AF
                expand!(OP_SRC_R_RBC, { t_src = (cpu.b as u16) << 8 | (cpu.c as u16); }); // source BC
                expand!(OP_SRC_R_RDE, { t_src = (cpu.d as u16) << 8 | (cpu.e as u16); }); // source DE
                expand!(OP_SRC_R_RHL, { t_src = (cpu.h as u16) << 8 | (cpu.l as u16); }); // source HL
                expand!(OP_SRC_R_RSP, { t_src = cpu.sp as u16; }); // source SP
                expand!(OP_SRC_R_RPC, { t_src = cpu.pc as u16; });
                //POP
                expand!(OP_POP_TO_SRC, { t_src = cpu.pop(mmu) as u16; });
                //
                //immediate
                expand!(OP_DST_D8, { t_dst = bytes[1] as u16; }); // source d8
                expand!(OP_DST_D16, { t_dst = (bytes[1] as u16) | ((bytes[2] as u16) << 8); }); // source d16
                //
                expand!(OP_DST_R_RA, { t_dst = cpu.a as u16; });
                expand!(OP_DST_R_RB, { t_dst = cpu.b as u16; });
                expand!(OP_DST_R_RC, { t_dst = cpu.c as u16; });
                expand!(OP_DST_R_RD, { t_dst = cpu.d as u16; });
                expand!(OP_DST_R_RE, { t_dst = cpu.e as u16; });
                expand!(OP_DST_R_RH, { t_dst = cpu.h as u16; });
                expand!(OP_DST_R_RL, { t_dst = cpu.l as u16; });
                expand!(OP_DST_R_RBC, { t_dst = (cpu.b as u16) << 8 | (cpu.c as u16); });
                expand!(OP_DST_R_RDE, { t_dst = (cpu.d as u16) << 8 | (cpu.e as u16); });
                expand!(OP_DST_R_RHL, { t_dst = (cpu.h as u16) << 8 | (cpu.l as u16); });
                expand!(OP_DST_R_RSP, { t_dst = cpu.sp as u16; });
                expand!(OP_DST_R_RPC, { t_dst = cpu.pc as u16; });
                //POP
                expand!(OP_POP_TO_DST, { t_dst = cpu.pop(mmu) as u16; });
                // OP_SRC_TO_AA
                expand!(OP_SRC_TO_AA, { t_abs_addr = t_src; });
                // OP_SRC_TO_RA
                expand!(OP_SRC_TO_RA, { t_rel_addr = t_src as i8; });
                // OP_DST_TO_AA
                expand!(OP_DST_TO_AA, { t_abs_addr = t_dst; } );
                // OP_DST_TO_RA
                expand!(OP_DST_TO_RA, { t_rel_addr = t_dst as i8; });

                // OP_COMPLEMENT
                //expand!(OP_EXAMPLE)
                expand!(OP_SRC_R_AA, { t_src = mmu.read_byte(t_abs_addr as usize) as u16; });
                expand!(OP_DST_R_AA, { t_dst = mmu.read_byte(t_abs_addr as usize) as u16; });
                expand!(OP_8BIT_AA, { t_abs_addr |= 0xFF00; } );
                // operations
                // should write to t_result generally, where we can write it however we want

                // OP_INC16
                expand!(OP_INC8, {
                    let (c, hc, result) = half_carry_add_u8(t_src as u8, 1);
                    if result == 0 {
                        zero = true;
                    }
                    half_carry = hc;
                    t_result = result as u16;
                });
                expand!(OP_XOR, { t_result = (cpu.a ^ (t_src as u8)) as u16; });
                expand!(OP_OR, { t_result = (cpu.a | (t_src as u8)) as u16; });
                expand!(OP_AND, { t_result = (cpu.a & (t_src as u8)) as u16; });
                expand!(OP_INC16, { t_result = t_src + 1; });
                expand!(OP_DEC8, {
                    let (c, hc, result) = half_carry_sub_u8(t_src as u8, 1);
                    if result == 0 {
                        zero = true;
                    }
                    half_carry = hc;
                    t_result = result as u16;
                });
                
                //
                expand!(OP_EXAMPLE, {
                    if cpu.get_flag(Flag::C) {
                        let (c, hc, temp) = half_carry_add_u8(t_dst as u8, 1);
                        t_dst = temp as u16;
                    }
                });
                expand!(OP_DEC16, { t_result = t_src - 1; });
                expand!(OP_APPLY_RA_TO_AA, { t_abs_addr = t_abs_addr.wrapping_add(t_rel_addr as u16); });


                

                expand!(OP_ADD_8BIT, {
                    let (c, h, res) = half_carry_add_u8(t_src as u8, t_dst as u8);
                    carry = c;
                    half_carry = h;
                    t_result = res as u16;
                });
                expand!(OP_SUB_8BIT, { let (c, h, res) = half_carry_sub_u8(t_src as u8, t_dst as u8);
                    carry = c;
                    half_carry = h;
                    t_result = res as u16; 
                });
                

                expand!(OP_SRC_TO_RESULT, { t_result = t_src; });
                expand!(OP_DST_TO_RESULT, { t_result = t_dst; });
                
                //PUSH
                expand!(OP_PUSH, { cpu.push(mmu, t_result); });

                expand!(OP_JUMP, { cpu.pc = t_abs_addr as u16; });
                expand!(OP_RES_W_ADDR_16, { mmu.write_word(t_abs_addr as usize, t_result); println!("16 bit write to {} {}", t_abs_addr, t_result);});
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
            },
        }; 
    }
}

/// Here this macro will expand in 0xFF of unique anonymous functions and we can just invoke these functions by indexing this array with the opcode
/// Inspired by Bisqwit's Nesemu1
type TimingHandler = fn(cpu: &LR35902) -> u8;
type OpHandler = fn(cpu: &mut LR35902, mmu: &mut MMU, bytes: [u8;4]);

macro_rules! cb_g {
    ($funname:ident, $opcode:literal, $inst_size:literal, $timing:literal) => {
        pub(super) const $funname: PendingOperation = PendingOperation{
            timing: 
                |cpu: & LR35902| 
        {
            let (op8, op8m) = op8_helper($opcode);
            let timing = $timing;
            macro_rules! expand {
               ($operation:expr, $code:block) => {
                    let op_block = op_block(&$operation, op8);
                    if (op_block & (op8m as u16)) > 0 $code
                }
            }
            timing
        },
            handler: 
                |cpu: &mut LR35902, mmu: &mut MMU, bytes: [u8;4]|
        
        {
            let (op8, op8m) = op8_helper($opcode);
            let mut t_src: u16 = 0;
            let mut t_dst: u16 = 0;
            let mut _enable_interrupts = false; let mut _interrupts_enabled = true; // should be on cpu struct most likely, if we have two instruction tables
            let mut _condition = false;
            let mut variety: u8 = variety_cb($opcode);

            // cpu.clocks.add($timing);
            cpu.pc += $inst_size;

            macro_rules! expand {
               ($operation:expr, $code:block) => {
                    let op_block = op_block(&$operation, op8);
                    if (op_block & (op8m as u16)) > 0 $code
                }
            }
            //expand!(&CBOP_VARIETY, { variety = ; });
            expand!(&CBOP_SRC_R_RA, { t_src = cpu.a as u16;});
            expand!(&CBOP_SRC_R_RB, { t_src = cpu.b as u16;});
            expand!(&CBOP_SRC_R_RC, { t_src = cpu.c as u16;});
            expand!(&CBOP_SRC_R_RD, { t_src = cpu.d as u16;});
            expand!(&CBOP_SRC_R_RE, { t_src = cpu.e as u16;});
            expand!(&CBOP_SRC_R_RH, { t_src = cpu.h as u16;});
            expand!(&CBOP_SRC_R_RL, { t_src = cpu.l as u16;});
            expand!(&CBOP_SRC_R_AHL, { t_src = mmu.read_byte(((cpu.h as usize) << 8) | cpu.l as usize)  as u16; });
            expand!(&CBOP_BIT, {
                let t = (t_src & (0x1 << variety)) > 0;
                if !t {
                        cpu.set_flag(Flag::Z);
                    }
                cpu.set_flag(Flag::H);
                cpu.reset_flag(Flag::N);
            });
        },
        };
    }
}

macro_rules! wrap_g {
    ($funname:ident, $opcode:literal, $inst_size:literal, $timing:literal) => {
        const $funname: PendingOperation = ($timing, $funname);
    };
}

#[cfg(test)]
mod tests;

#[cfg(feature = "instruction_table")]
mod instructions;