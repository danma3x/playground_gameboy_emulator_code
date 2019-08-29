pub struct MMU {
    ram: Vec<u8>,
}

impl MMU {
    pub fn new() -> MMU {
        let ram= vec![0; 65_536];
        MMU {
            ram
        }
    }
    pub fn read_ahead(&self, at: usize) -> [u8; 4] {
        let mut ret: [u8;4] = Default::default();
        ret.copy_from_slice(&self.ram[at..at+4]);
        ret
    }

    pub fn read_word(&self, at: usize) -> u16 {
        let word = ((self.ram[at] as u16) << 8) + (self.ram[at+1] as u16);
        word
    }

    pub fn read_byte(&self, at: usize) -> u8 {
        self.ram[at]
    }

    pub fn write_byte(&mut self, at: usize, byte: u8) {
        self.ram[at] = byte;
    }

    pub fn write_word(&mut self, at: usize, word: u16) {
        unimplemented!()
    }

    pub fn initialize<I>(&mut self, iter: I)
        where I: IntoIterator<Item=u8> {
        for (i, byte) in iter.into_iter().enumerate() {
            self.write_byte(i, byte);
        }
    }
}


