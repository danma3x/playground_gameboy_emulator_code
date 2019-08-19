use gameboy_emulator::mmu::MMU;
use gameboy_emulator::cpu::LR35902;
use std::fs::File;
use std::io::{Read};


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut mmu  = {
        let mut mmu = MMU::new();
        let mut file = File::open("/home/danmatrix/DMG_ROM.bin").expect("Couldn't find the file");
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).expect("Failed to read the file");
        mmu.initialize(buf);
        mmu
    };

    let mut cpu = LR35902::new();
    for _ in 0..5 {
        cpu.execute(&mut mmu);
        println!("CPU state: {:?}", cpu);
    }
    Ok(())
}