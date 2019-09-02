use memory_bus::MMU;
use lr35902::LR35902;
use ppu::{PPU, PPUWindow};

use std::fs::File;
use std::io::{Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut mmu  = {
        let mut mmu = MMU::new();
        let mut file = File::open("D:/hobby/DMG_ROM.bin").expect("Couldn't find the file");
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).expect("Failed to read the file");
        mmu.initialize(buf);
        mmu
    };

    let out = std::io::stdout();
    let lock = out.lock();
    let mut w = std::io::BufWriter::new(lock);

    let mut cpu = LR35902::new();
    cpu.init(&mut mmu);
    let mut ppu_window = PPUWindow::new();
    'update_loop: loop {
        cpu.step(&mut mmu, 2);
        ppu_window.ppu.step(&mmu, cpu.clocks.current as usize);
        cpu.step(&mut mmu, 2);
        if !ppu_window.update() { break 'update_loop; }
        if cpu.clocks.total > 4_000_000 {
            break 'update_loop;
        }
    }

    // for _ in 0..1_000_000 {
    //     println!("{:?}", cpu);
    //     cpu.execute(&mut mmu);
    // }
    Ok(())
}