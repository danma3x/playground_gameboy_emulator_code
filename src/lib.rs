pub mod mmu;
pub mod cpu;
// pub mod old_cpu;

pub fn test_function() {
    let mut cpu = cpu::LR35902::new();
    let mut mmu = mmu::MMU::new();
    cpu::INS_TABLE[0x31](&mut cpu, &mut mmu, [0x00, 0x00, 0x00, 0x00]);
}