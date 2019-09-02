use minifb::{Key, WindowOptions, Window};
use memory_bus::MMU;

pub type Clocks = usize;
pub type Lines = usize;


#[derive(Debug)]
enum PPUMode {
    ScanlineOAM,
    ScanlineVRAM,
    HBlank,
    VBlank,
}

impl Default for PPUMode {
    fn default() -> PPUMode {
        PPUMode::ScanlineOAM
    }
}

#[derive(Default, Debug)]
pub struct PPU {
    mode: PPUMode,
    clocks: Clocks,
    lines: Lines,
}

impl PPU {
    pub fn step(&mut self, mmu: &MMU, deltaclock: usize) {
        self.clocks = self.clocks + deltaclock;
        match self.mode {
            PPUMode::ScanlineOAM => {
                if self.clocks >= 80 {
                    self.mode = PPUMode::ScanlineVRAM;
                    self.clocks -= 80;
                }
            }
            PPUMode::ScanlineVRAM => {
                if self.clocks >= 172 {
                    self.mode = PPUMode::HBlank;
                    self.clocks -= 172;
                }
            }
            PPUMode::HBlank => {
                if self.clocks >= 204 {
                    self.mode = PPUMode::ScanlineOAM;
                    self.clocks -= 204;
                    self.lines += 1;
                    if self.lines == 143 {
                        self.mode = PPUMode::VBlank;
                        //VBLANK handling
                    }
                }

            }
            PPUMode::VBlank => {
                if self.clocks >= 456 {
                    self.mode = PPUMode::ScanlineOAM;
                    self.clocks -= 456;
                    self.lines += 1;
                    if self.lines > 153 {
                        self.mode = PPUMode::ScanlineOAM;
                        self.lines = 0;
                    }
                }
            }
        }
    }

    pub fn render_background(&mut self, mmu: &mut MMU) {
        let cram =  &mmu.ram[0x8000..0x9800];
        let bmap_1 = &mmu.ram[0x9C00..0xA000];
    }
}

pub struct PPUWindow {
    pub ppu: PPU,
    buffer: Vec<u32>,
    window: Window,
}

impl PPUWindow {
    pub fn new() -> PPUWindow {
        let window = Window::new("Test", 640, 480, WindowOptions::default()).unwrap_or_else(|e| panic!("{}, e"));
        let buffer: Vec<u32> = vec![0; 640 * 480];
        let ppu = Default::default();
        PPUWindow{ window, buffer, ppu }
        // while window.is_open() && !window.is_key_down(Key::Escape) {
        //     window.update_with_buffer(&buffer).unwrap();
        // }
    }

    pub fn update(&mut self) -> bool {
        let mut is_updated = self.window.is_open() && !self.window.is_key_down(Key::Escape);
        if is_updated {
            self.window.update();
        }
        is_updated
    }
}