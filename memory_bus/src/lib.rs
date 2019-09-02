pub struct CharacterRAM {
    tiles: Vec<u8>,
}

impl Default for CharacterRAM {
    fn default() -> CharacterRAM {
        CharacterRAM {
            tiles: Vec::with_capacity(0x1800)
        }
    }
}

pub struct BGMapData {
    tile_index: Vec<u8>,
}

impl Default for BGMapData {
    fn default() -> BGMapData {
        BGMapData {
            tile_index: Vec::with_capacity(0xFF)
        }
    }
}

pub struct MMU {
    pub ram: Vec<u8>,
    // pub bgmapdata1: BGMapData,
    // pub bgmapdata2: BGMapData,
    // pub cram: CharacterRAM,
}

impl MMU {
    pub fn new() -> MMU {
        let ram = vec![0;0x10000];
        MMU {
            ram
        }
    }
    pub fn read_ahead(&self, at: usize) -> [u8; 4] {
        let mut ret: [u8;4] = Default::default();
        ret.copy_from_slice(&self.ram[at..at+4]);
        ret
    }

    pub fn read_word(&self, at: usize) -> u16 { // le
        let word = (self.ram[at] as u16) | ((self.ram[at+1] as u16) << 8);
        word
    }

    pub fn read_byte(&self, at: usize) -> u8 {
        self.ram[at]
    }

    pub fn write_byte(&mut self, at: usize, byte: u8) {
        self.ram[at] = byte;
    }

    pub fn write_word(&mut self, at: usize, word: u16) { // le
        self.ram[at] = word as u8;
        self.ram[at+1] = (word >> 8) as u8;
    }

    pub fn initialize<I>(&mut self, iter: I)
        where I: IntoIterator<Item=u8> {
        for (i, byte) in iter.into_iter().enumerate() {
            self.write_byte(i, byte);
        }
    }
}


