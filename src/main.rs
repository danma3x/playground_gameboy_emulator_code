use gameboy_emulator::mmu::MMU;
use gameboy_emulator::cpu::LR35902;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cpu = LR35902::new();
    cpu.execute(0x02);
    Ok(())
}