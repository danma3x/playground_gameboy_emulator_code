use memory_bus::MMU;
use lr35902::LR35902;
use std::fs::File;
use std::io::{Read};




fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut mmu  = {
        let mut mmu = MMU::new();
        let mut file = File::open("D:/hobby/DMG_ROM.bin").expect("Couldn't find the file");
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).expect("Failed to read the file");
        mmu.initialize(buf);
        mmu
    };

    let mut cpu = LR35902::new();
    cpu.execute(&mut mmu);

    for _ in 0..5 {
        cpu.execute(&mut mmu);
        println!("{}", cpu.pc);
    }
    Ok(())
}