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
    pub sp: u16,
    pub t: usize,
}

const fn op8_helper(opcode: u8) -> (usize, u64) {
    ((opcode / 64) as usize, (1 << (opcode % 64)) as u64)
}

/// Opcode processing and all cpu operation is actually described here, because this macro will codegen a unique function for each opcode and all the IF statements should be compiled away.
/// Not much code here yet, but we should be able all of the possible operations here and get away without severe performance penalty thanks to compile time optimisations and the final code shouldn't be too hard, though 0xCB prefix will be a headache I guess
macro_rules! generate {
    ($opcode:literal) => {
        |cpu|
        {
            let (op8, op8m) = op8_helper($opcode);
            let mut temp: u64 = 0;
            macro_rules! expand {
               ($x:expr, $c:block) => {
                    let op_block = $x[op8];
                    if (op_block & op8m) > 0 $c
                }
            }
            expand!(OPERATION_LOAD_REGISTER_A, {
                temp = cpu.a as u64;
                println!("Load operation invoked");
            });
            expand!(OPERATION_STORE_REGISTER_A, {
                cpu.a = temp as u8;
                println!("Store operation invoked");
            });
            println!("Opcode is {}", $opcode);
        }
    }
}

/// Here this macro will expand in 0xFF of unique anonymous functions and we can just invoke these functions by indexing this array with the opcode
/// Inspired by Bisqwit's Nesemu1
const INS_TABLE: [fn(cpu: &mut LR35902); 256] = [
    generate!(0x00),generate!(0x01),generate!(0x02),generate!(0x03),generate!(0x04),generate!(0x05),generate!(0x06),generate!(0x07),generate!(0x08),generate!(0x09),generate!(0x0A),generate!(0x0B),generate!(0x0C),generate!(0x0D),generate!(0x0E),generate!(0x0F),
    generate!(0x10),generate!(0x11),generate!(0x12),generate!(0x13),generate!(0x14),generate!(0x15),generate!(0x16),generate!(0x17),generate!(0x18),generate!(0x19),generate!(0x1A),generate!(0x1B),generate!(0x1C),generate!(0x1D),generate!(0x1E),generate!(0x1F),
    generate!(0x20),generate!(0x21),generate!(0x22),generate!(0x23),generate!(0x24),generate!(0x25),generate!(0x26),generate!(0x27),generate!(0x28),generate!(0x29),generate!(0x2A),generate!(0x2B),generate!(0x2C),generate!(0x2D),generate!(0x2E),generate!(0x2F),
    generate!(0x30),generate!(0x31),generate!(0x32),generate!(0x33),generate!(0x34),generate!(0x35),generate!(0x36),generate!(0x37),generate!(0x38),generate!(0x39),generate!(0x3A),generate!(0x3B),generate!(0x3C),generate!(0x3D),generate!(0x3E),generate!(0x3F),
    generate!(0x40),generate!(0x41),generate!(0x42),generate!(0x43),generate!(0x44),generate!(0x45),generate!(0x46),generate!(0x47),generate!(0x48),generate!(0x49),generate!(0x4A),generate!(0x4B),generate!(0x4C),generate!(0x4D),generate!(0x4E),generate!(0x4F),
    generate!(0x50),generate!(0x51),generate!(0x52),generate!(0x53),generate!(0x54),generate!(0x55),generate!(0x56),generate!(0x57),generate!(0x58),generate!(0x59),generate!(0x5A),generate!(0x5B),generate!(0x5C),generate!(0x5D),generate!(0x5E),generate!(0x5F),
    generate!(0x60),generate!(0x61),generate!(0x62),generate!(0x63),generate!(0x64),generate!(0x65),generate!(0x66),generate!(0x67),generate!(0x68),generate!(0x69),generate!(0x6A),generate!(0x6B),generate!(0x6C),generate!(0x6D),generate!(0x6E),generate!(0x6F),
    generate!(0x70),generate!(0x71),generate!(0x72),generate!(0x73),generate!(0x74),generate!(0x75),generate!(0x76),generate!(0x77),generate!(0x78),generate!(0x79),generate!(0x7A),generate!(0x7B),generate!(0x7C),generate!(0x7D),generate!(0x7E),generate!(0x7F),
    generate!(0x80),generate!(0x81),generate!(0x82),generate!(0x83),generate!(0x84),generate!(0x85),generate!(0x86),generate!(0x87),generate!(0x88),generate!(0x89),generate!(0x8A),generate!(0x8B),generate!(0x8C),generate!(0x8D),generate!(0x8E),generate!(0x8F),
    generate!(0x90),generate!(0x91),generate!(0x92),generate!(0x93),generate!(0x94),generate!(0x95),generate!(0x96),generate!(0x97),generate!(0x98),generate!(0x99),generate!(0x9A),generate!(0x9B),generate!(0x9C),generate!(0x9D),generate!(0x9E),generate!(0x9F),
    generate!(0xA0),generate!(0xA1),generate!(0xA2),generate!(0xA3),generate!(0xA4),generate!(0xA5),generate!(0xA6),generate!(0xA7),generate!(0xA8),generate!(0xA9),generate!(0xAA),generate!(0xAB),generate!(0xAC),generate!(0xAD),generate!(0xAE),generate!(0xAF),
    generate!(0xB0),generate!(0xB1),generate!(0xB2),generate!(0xB3),generate!(0xB4),generate!(0xB5),generate!(0xB6),generate!(0xB7),generate!(0xB8),generate!(0xB9),generate!(0xBA),generate!(0xBB),generate!(0xBC),generate!(0xBD),generate!(0xBE),generate!(0xBF),
    generate!(0xC0),generate!(0xC1),generate!(0xC2),generate!(0xC3),generate!(0xC4),generate!(0xC5),generate!(0xC6),generate!(0xC7),generate!(0xC8),generate!(0xC9),generate!(0xCA),generate!(0xCB),generate!(0xCC),generate!(0xCD),generate!(0xCE),generate!(0xCF),
    generate!(0xD0),generate!(0xD1),generate!(0xD2),generate!(0xD3),generate!(0xD4),generate!(0xD5),generate!(0xD6),generate!(0xD7),generate!(0xD8),generate!(0xD9),generate!(0xDA),generate!(0xDB),generate!(0xDC),generate!(0xDD),generate!(0xDE),generate!(0xDF),
    generate!(0xE0),generate!(0xE1),generate!(0xE2),generate!(0xE3),generate!(0xE4),generate!(0xE5),generate!(0xE6),generate!(0xE7),generate!(0xE8),generate!(0xE9),generate!(0xEA),generate!(0xEB),generate!(0xEC),generate!(0xED),generate!(0xEE),generate!(0xEF),
    generate!(0xF0),generate!(0xF1),generate!(0xF2),generate!(0xF3),generate!(0xF4),generate!(0xF5),generate!(0xF6),generate!(0xF7),generate!(0xF8),generate!(0xF9),generate!(0xFA),generate!(0xFB),generate!(0xFC),generate!(0xFD),generate!(0xFE),generate!(0xFF)
];


const OPERATION_EXAMPLE: [u64; 4] = [
    0b00000000_00000100__00000000_00000100___00000000_00000100__00000000_00000100,
    0b00000000_00000000__00000000_00000000___00000000_00000000__00000000_00000000,
    0b00000000_00000000__00000000_00000000___00000000_00000000__00000000_00000000,
    0b00000000_00000000__00000000_00000000___00000000_00000000__00000000_00000000];

const OPERATION_LOAD_REGISTER_A: [u64; 4] = [
    0b00000000_00000100__00000000_00000100___00000000_00000100__00000000_00000100,
    0b10000000_10000000__10000000_10000000___10000000_10000000__10000000_10000000,
    0b10000000_10000000__10000000_10000000___10000000_10000000__10000000_10000000,
    0b00000000_00000000__00000000_00000000___00000000_00000000__00000000_00000000];

const OPERATION_STORE_REGISTER_A: [u64; 4] = [
    0b00000000_00000000__00000000_00000000___00000000_00000000__00000000_00000000,
    0b00000000_00000000__00000000_00000000___00000000_00000000__00000000_00000000,
    0b00000000_00000000__00000000_00000000___00000000_00000000__00000000_00000000,
    0b00000000_00000000__00000000_00000000___00000000_00000000__00000000_00000000];

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
    pub fn execute(&mut self, opcode: u8) {
        let mut temp: u64 = 0;
        // I'll probably read 4 bytes here and send them to the opcode function
        INS_TABLE[opcode as usize](self);

    }
}