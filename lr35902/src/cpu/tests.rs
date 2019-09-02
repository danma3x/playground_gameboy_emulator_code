use super::*;

fn prerequisites() -> ( LR35902, MMU ){
    ( LR35902::new(), MMU::new() )
}

#[test]
fn nop_test() {
    let (mut cpu, mut mmu) = prerequisites();
    g!(f0, 0x00, 1, 4); // nop
    (f0.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn ld_test() {
    let (mut cpu, mut mmu) = prerequisites();
    g!(ld_bc_d16, 0x01, 3, 12); // bc, d16
    (ld_bc_d16.handler)(&mut cpu, &mut mmu, [0x0, 0x01, 0x30, 0x0]);
    assert_eq!(cpu.b, 0x30);
    assert_eq!(cpu.c, 0x01);

    g!(ld_de_d16, 0x11, 3, 12); // de, d16
    (ld_de_d16.handler)(&mut cpu, &mut mmu, [0x0, 0x01, 0x30, 0x0]);
    assert_eq!(cpu.d, 0x30);
    assert_eq!(cpu.e, 0x01);

    g!(ld_hl_d16, 0x21, 3, 12); // hl, d16
    (ld_hl_d16.handler)(&mut cpu, &mut mmu, [0x0, 0x01, 0x30, 0x0]);
    assert_eq!(cpu.h, 0x30);
    assert_eq!(cpu.l, 0x01);

    g!(ld_sp_d16, 0x31, 3, 12); // sp, d16
    (ld_sp_d16.handler)(&mut cpu, &mut mmu, [0x0, 0x01, 0x30, 0x0]);
    assert_eq!(cpu.sp, 0x3001);

    g!(ld_abc_a, 0x02, 1, 8);
    cpu.b = 0x1; cpu.c = 0x0;
    cpu.a = 0x50;
    (ld_abc_a.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(mmu.read_byte(0x100), 0x50);

    g!(ld_ade_a, 0x12, 1, 8);
    cpu.d = 0x2; cpu.e = 0x0;
    cpu.a = 0x50;
    (ld_ade_a.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(mmu.read_byte(0x200), 0x50);

    g!(ld_ahlinc_a, 0x22, 1, 8);

    cpu.h = 0x3; cpu.l = 0x0;
    cpu.a = 0x50;
    (ld_ahlinc_a.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(mmu.read_byte(0x300), 0x50);
    assert_eq!(cpu.h, 0x3); assert_eq!(cpu.l, 0x1);

    g!(ld_ahldec_a, 0x32, 1, 8);
    cpu.h = 0x4; cpu.l = 0x0;
    cpu.a = 0x50;
    (ld_ahldec_a.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(mmu.read_byte(0x400), 0x50);
    assert_eq!(cpu.h, 0x3); assert_eq!(cpu.l, 0xFF);

    g!(ld_b_d8, 0x06, 2, 8);
    (ld_b_d8.handler)(&mut cpu, &mut mmu, [0x0, 0x18, 0x0, 0x0]);
    assert_eq!(cpu.b, 0x18);

    g!(ld_d_d8, 0x16, 2, 8);
    (ld_d_d8.handler)(&mut cpu, &mut mmu, [0x0, 0x18, 0x0, 0x0]);
    assert_eq!(cpu.d, 0x18);

    g!(ld_h_d8, 0x26, 2, 8);
    (ld_h_d8.handler)(&mut cpu, &mut mmu, [0x0, 0x18, 0x0, 0x0]);
    assert_eq!(cpu.h, 0x18);


    g!(ld_ahl_d8, 0x36, 2, 8);
    cpu.h = 0x5;
    cpu.l = 0x0;
    (ld_ahl_d8.handler)(&mut cpu, &mut mmu, [0x0, 0x18, 0x0, 0x0]);
    assert_eq!(mmu.read_byte(0x500), 0x18);

    g!(ld_a16_sp, 0x08, 3, 20);
    cpu.sp = 0xFF00;
    (ld_a16_sp.handler)(&mut cpu, &mut mmu, [0x0, 0x50, 0x10, 0x0]);
    assert_eq!(mmu.read_word(0x1050), 0xFF00);
    
    {
        g!(ld_a_abc, 0x0A, 1, 8);
        cpu.b = 0x10;
        cpu.c = 0x50;
        mmu.write_byte(0x1050, 0xF0);
        (ld_a_abc.handler)(&mut cpu, &mut mmu, [0x0, 0x50, 0x10, 0x0]);
        assert_eq!(cpu.a, 0xF0);

        g!(ld_a_ade, 0x1A, 1, 8);
        cpu.d = 0x10;
        cpu.e = 0x50;
        mmu.write_byte(0x1050, 0xF1);
        (ld_a_ade.handler)(&mut cpu, &mut mmu, [0x0, 0x50, 0x10, 0x0]);
        assert_eq!(cpu.a, 0xF1);

        g!(ld_a_ahlinc, 0x2A, 1, 8);
        cpu.h = 0x10;
        cpu.l = 0x50;
        mmu.write_byte(0x1050, 0xF0);
        (ld_a_ahlinc.handler)(&mut cpu, &mut mmu, [0x0, 0x50, 0x10, 0x0]);
        assert_eq!(cpu.a, 0xF0);
        assert_eq!(cpu.h, 0x10);
        assert_eq!(cpu.l, 0x51);

        g!(ld_a_ahldec, 0x3A, 1, 8);
        cpu.h = 0x10;
        cpu.l = 0x50;
        mmu.write_byte(0x1050, 0xF2);
        (ld_a_ahldec.handler)(&mut cpu, &mut mmu, [0x0, 0x50, 0x10, 0x0]);
        assert_eq!(cpu.a, 0xF2);
        assert_eq!(cpu.h, 0x10);
        assert_eq!(cpu.l, 0x4F);

    }

    {
        g!(ld_c_d8, 0x0E, 2, 8);
        cpu.c = 0;
        (ld_c_d8.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.c, 0x10);

        g!(ld_e_d8, 0x1E, 2, 8);
        cpu.e = 0;
        (ld_e_d8.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.e, 0x10);

        g!(ld_l_d8, 0x2E, 2, 8);
        cpu.l = 0;
        (ld_l_d8.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.l, 0x10);

        g!(ld_a_d8, 0x3E, 2, 8);
        cpu.a = 0;
        (ld_a_d8.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.a, 0x10);
    }

    {
        g!(ld_b_b, 0x40, 1, 4);
        cpu.b = 1;
        (ld_b_b.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.b, 1);

        g!(ld_b_c, 0x41, 1, 4);
        cpu.c = 2;
        (ld_b_c.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.b, 2);

        g!(ld_b_d, 0x42, 1, 4);
        cpu.d = 3;
        (ld_b_d.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.b, 3);

        g!(ld_b_e, 0x43, 1, 4);
        cpu.e = 4;
        (ld_b_e.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.b, 4);

        g!(ld_b_h, 0x44, 1, 4);
        cpu.h = 5;
        (ld_b_h.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.b, 5);

        g!(ld_b_l, 0x45, 1, 4);
        cpu.l = 6;
        (ld_b_l.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.b, 6);

        //

        g!(ld_d_b, 0x50, 1, 4);
        cpu.b = 5;
        (ld_d_b.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.d, 5);

        g!(ld_d_c, 0x51, 1, 4);
        cpu.c = 6;
        (ld_d_c.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.d, 6);

        g!(ld_d_d, 0x52, 1, 4);
        cpu.d = 7;
        (ld_d_d.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.d, 7);

        g!(ld_d_e, 0x53, 1, 4);
        cpu.e = 8;
        (ld_d_e.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.d, 8);

        g!(ld_d_h, 0x54, 1, 4);
        cpu.h = 9;
        (ld_d_h.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.d, 9);

        g!(ld_d_l, 0x55, 1, 4);
        cpu.l = 10;
        (ld_d_l.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.d, 10);
    }

    {
        g!(ld_b_ahl, 0x46, 1, 4);
        cpu.h = 0x2;
        cpu.l = 0x0;
        mmu.write_byte(0x200, 0x25);
        cpu.b = 0;
        (ld_b_ahl.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.b, 0x25);

        g!(ld_d_ahl, 0x56, 1, 4);
        cpu.h = 0x3;
        cpu.l = 0x0;
        mmu.write_byte(0x300, 0x35);
        cpu.d = 0;
        (ld_d_ahl.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.d, 0x35);

        g!(ld_h_ahl, 0x66, 1, 4);
        cpu.h = 0x4;
        cpu.l = 0x0;
        mmu.write_byte(0x400, 0x55);
        (ld_h_ahl.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.h, 0x55);

        g!(ld_c_ahl, 0x4E, 1, 4);
        cpu.h = 0x2;
        cpu.l = 0x1;
        mmu.write_byte(0x201, 0x15);
        (ld_c_ahl.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.c, 0x15);

        g!(ld_e_ahl, 0x5E, 1, 4);
        cpu.h = 0x3;
        cpu.l = 0x1;
        mmu.write_byte(0x301, 0x66);
        (ld_e_ahl.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.e, 0x66);

        g!(ld_l_ahl, 0x6E, 1, 4);
        cpu.h = 0x4;
        cpu.l = 0x1;
        mmu.write_byte(0x401, 0x76);
        (ld_l_ahl.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.l, 0x76);
        
        g!(ld_a_ahl, 0x7E, 1, 4);
        cpu.h = 0x5;
        cpu.l = 0x1;
        mmu.write_byte(0x501, 0x86);
        (ld_a_ahl.handler)(&mut cpu, &mut mmu, [0x0, 0x10, 0x0, 0x0]);
        assert_eq!(cpu.a, 0x86);
    }
//        let addr = 0x6000;
//        let b = 0x5B5B;
//        cpu.sp = b;
//        mmu.write_byte(addr, b);
//        ld_a16_sp(&mut cpu, &mut mmu, [0x0, 0x0, 0x60, 0x0]);
//        assert_eq!(mmu.read_byte(addr), )
}

#[test]
fn inc_dec_test() {
    let (mut cpu, mut mmu) = prerequisites();
    
    g!(inc_sp, 0x33, 1, 8);
    g!(inc_b, 0x04, 1, 4);
    g!(inc_d, 0x14, 1, 4);
    g!(inc_h, 0x24, 1, 4);
    g!(inc_ahl, 0x34, 1, 12);
    g!(dec_b, 0x05, 1, 4);
    g!(dec_d, 0x15, 1, 4);
    g!(dec_h, 0x25, 1, 4);
    g!(dec_ahl, 0x35, 1, 12);

    g!(dec_bc, 0x0B, 1, 8);
    g!(dec_de, 0x1B, 1, 8);
    g!(dec_hl, 0x2B, 1, 8);
    g!(dec_sp, 0x3B, 1, 8);
    g!(inc_c, 0x0C, 1, 4);
    g!(inc_e, 0x1C, 1, 4);
    g!(inc_l, 0x2C, 1, 4);
    g!(inc_a, 0x3C, 1, 4);
    g!(dec_c, 0x0D, 1, 4);
    g!(dec_e, 0x1D, 1, 4);
    g!(dec_l, 0x2D, 1, 4);
    g!(dec_a, 0x3D, 1, 4);

    g!(inc_bc, 0x03, 1, 8); // bc, d16
    cpu.b=0x00;
    cpu.c=0xFF;
    (inc_bc.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.b, 0x1);
    assert_eq!(cpu.c, 0x0);

    g!(inc_de, 0x13, 1, 8);
    cpu.d=0x00;
    cpu.e=0xFF;
    (inc_de.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.d, 0x1);
    assert_eq!(cpu.e, 0x0);

    g!(inc_hl, 0x23, 1, 8);
    cpu.h=0x00;
    cpu.l=0xFF;
    (inc_hl.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.h, 0x1);
    assert_eq!(cpu.l, 0x0);

    cpu.sp = 0x00FF;
    (inc_sp.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.sp, 0x0100);

    cpu.b=0x01;
    cpu.c=0x00;
    (dec_bc.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.b, 0x0);
    assert_eq!(cpu.c, 0xFF);

    cpu.d=0x01;
    cpu.e=0x00;
    (dec_de.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.d, 0x0);
    assert_eq!(cpu.e, 0xFF);

    cpu.h=0x01;
    cpu.l=0x00;
    (dec_hl.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.h, 0x00);
    assert_eq!(cpu.l, 0xFF);

    cpu.sp = 0x0100;
    (dec_sp.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.sp, 0x00FF);

    cpu.reset_flag(Flag::H);
    cpu.b = 0x0F;
    (inc_b.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.b, 0x10);
    assert_eq!(cpu.get_flag(Flag::H), true);

    cpu.reset_flag(Flag::H);
    cpu.d = 0x0F;
    (inc_d.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.d, 0x10);
    assert_eq!(cpu.get_flag(Flag::H), true);

    cpu.reset_flag(Flag::H);
    cpu.h = 0x0F;
    (inc_h.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.h, 0x10);
    assert_eq!(cpu.get_flag(Flag::H), true);

    cpu.reset_flag(Flag::H);
    cpu.h = 0x2;
    cpu.l = 0x0;
    (inc_ahl.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(mmu.read_byte(0x200), 0x01);
    assert_eq!(cpu.get_flag(Flag::H), false);

    //dec8

    cpu.reset_flag(Flag::H);
    cpu.b = 0x10;
    (dec_b.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.b, 0x0F);
    assert_eq!(cpu.get_flag(Flag::H), true);

    cpu.reset_flag(Flag::H);
    cpu.d = 0x10;
    (dec_d.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.d, 0x0F);
    assert_eq!(cpu.get_flag(Flag::H), true);

    cpu.reset_flag(Flag::H);
    cpu.h = 0x10;
    (dec_h.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.h, 0x0F);
    assert_eq!(cpu.get_flag(Flag::H), true);

    cpu.reset_flag(Flag::H);
    cpu.h = 0x9;
    cpu.l = 0x0;
    mmu.write_byte(0x900, 255);
    (dec_ahl.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(mmu.read_byte(0x900), 0xFE);
    assert_eq!(cpu.get_flag(Flag::H), false);

    cpu.reset_flag(Flag::H);
    cpu.c = 0x0F;
    (inc_c.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.c, 0x10);
    assert_eq!(cpu.get_flag(Flag::H), true);

    cpu.reset_flag(Flag::H);
    cpu.c = 0x0F;
    (inc_c.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.c, 0x10);
    assert_eq!(cpu.get_flag(Flag::H), true);

    cpu.reset_flag(Flag::H);
    cpu.e = 0x0F;
    (inc_e.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.e, 0x10);
    assert_eq!(cpu.get_flag(Flag::H), true);

    cpu.reset_flag(Flag::H);
    cpu.l = 0x0F;
    (inc_l.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.l, 0x10);
    assert_eq!(cpu.get_flag(Flag::H), true);

    cpu.reset_flag(Flag::H);
    cpu.a = 0x0F;
    (inc_a.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.a, 0x10);
    assert_eq!(cpu.get_flag(Flag::H), true);

    cpu.reset_flag(Flag::H);
    cpu.c = 0x10;
    (dec_c.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.c, 0x0F);
    assert_eq!(cpu.get_flag(Flag::H), true);

    cpu.reset_flag(Flag::H);
    cpu.e = 0x10;
    (dec_e.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.e, 0x0F);
    assert_eq!(cpu.get_flag(Flag::H), true);

    cpu.reset_flag(Flag::H);
    cpu.l = 0x10;
    (dec_l.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.l, 0x0F);
    assert_eq!(cpu.get_flag(Flag::H), true);

    cpu.reset_flag(Flag::H);
    cpu.a = 0x10;
    (dec_a.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.a, 0x0F);
    assert_eq!(cpu.get_flag(Flag::H), true);
}

#[test]
fn jump_test() {
    let (mut cpu, mut mmu) = prerequisites();
    g!(jr_nz_r8, 0x20, 2, 12);
    g!(jr_nc_r8, 0x30, 2, 12);
    g!(jr_r8, 0x18, 2, 12);
    g!(jr_z_r8, 0x28, 2, 12);
    g!(jr_c_r8, 0x38, 2, 12);

    cpu.reset_flag(Flag::Z);
    cpu.pc = 0xFF;
    (jr_nz_r8.handler)(&mut cpu, &mut mmu, [0x0, (0-5) as u8, 0x0, 0x0]);
    assert_eq!(cpu.pc, 0xFF - 5 + 2);

    cpu.reset_flag(Flag::C);
    cpu.pc = 0xFF;
    (jr_nc_r8.handler)(&mut cpu, &mut mmu, [0x0, (0-5) as u8, 0x0, 0x0]);
    assert_eq!(cpu.pc, 0xFF - 5 + 2);

    cpu.pc = 0xFF;
    (jr_r8.handler)(&mut cpu, &mut mmu, [0x0, (0-5) as u8, 0x0, 0x0]);
    assert_eq!(cpu.pc, 0xFF - 5 + 2);

    cpu.reset_flag(Flag::Z);
    cpu.pc = 0xFF;
    (jr_z_r8.handler)(&mut cpu, &mut mmu, [0x0, (0-5) as u8, 0x0, 0x0]);
    assert_eq!(cpu.pc, 0x0101);
    cpu.set_flag(Flag::Z);
    (jr_z_r8.handler)(&mut cpu, &mut mmu, [0x0, (0-5) as u8, 0x0, 0x0]);
    assert_eq!(cpu.pc, 0x0101 - 5 + 2);

    cpu.reset_flag(Flag::C);
    cpu.pc = 0xFF;
    (jr_c_r8.handler)(&mut cpu, &mut mmu, [0x0, (0-5) as u8, 0x0, 0x0]);
    assert_eq!(cpu.pc, 0x0101);
    cpu.set_flag(Flag::C);
    (jr_c_r8.handler)(&mut cpu, &mut mmu, [0x0, (0-5) as u8, 0x0, 0x0]);
    assert_eq!(cpu.pc, 0x0101 - 5 + 2);
}

#[test]
fn xor_test() {
    let (mut cpu, mut mmu) = prerequisites();
    g!(xor_b, 0xA8, 1, 4);
    cpu.a = 0b1010_1010;
    cpu.b = 0b1111_1111;
    (xor_b.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.a, 0b0101_0101);

    g!(xor_c, 0xA9, 1, 4);
    g!(xor_d, 0xAA, 1, 4);
    g!(xor_e, 0xAB, 1, 4);
    g!(xor_h, 0xAC, 1, 4);
    g!(xor_l, 0xAD, 1, 4);
    g!(xor_ahl, 0xAE, 1, 4);
    g!(xor_a, 0xAF, 1, 4);


}

#[test]
fn and_test() {
    let (mut cpu, mut mmu) = prerequisites();
    g!(and_b, 0xA0, 1, 4);
    cpu.a = 0b1111_0000;
    cpu.b = 0b0101_0101;
    (and_b.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.a, 0b0101_0000);
}

#[test]
fn or_test() {
    let (mut cpu, mut mmu) = prerequisites();
    g!(or_b, 0xB0, 1, 4);
    cpu.a = 0b1010_1010;
    cpu.b = 0b0000_1111;
    (or_b.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.a, 0b1010_1111);
}

#[test]
fn cb_bit_test() {
    let (mut cpu, mut mmu) = prerequisites();
    cb_g!(bit_0_b, 0x40, 2, 8);
    cb_g!(bit_0_c, 0x41, 2, 8);
    cb_g!(bit_0_d, 0x42, 2, 8);
    cb_g!(bit_0_e, 0x43, 2, 8);
    cb_g!(bit_0_h, 0x44, 2, 8);
    cb_g!(bit_0_l, 0x45, 2, 8);
    cb_g!(bit_0_ahl, 0x46, 2, 8);
    cb_g!(bit_0_a, 0x47, 2, 8);

    cpu.reset_flag(Flag::Z);
    (bit_0_b.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.get_flag(Flag::Z), true);
    cpu.reset_flag(Flag::Z);
    (bit_0_c.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.get_flag(Flag::Z), true);
    cpu.reset_flag(Flag::Z);
    (bit_0_d.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.get_flag(Flag::Z), true);
    cpu.reset_flag(Flag::Z);
    (bit_0_e.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.get_flag(Flag::Z), true);
    cpu.reset_flag(Flag::Z);
    (bit_0_h.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.get_flag(Flag::Z), true);
    cpu.reset_flag(Flag::Z);
    (bit_0_l.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.get_flag(Flag::Z), true);
    cpu.reset_flag(Flag::Z);
    (bit_0_ahl.handler)(&mut cpu, &mut mmu, [0x0, 0x0, 0x0, 0x0]);
    assert_eq!(cpu.get_flag(Flag::Z), true);



}

#[test]
fn call_ret_test() {
    let (mut cpu, mut mmu) = prerequisites();
    cpu.sp = 0xFFFF;
    g!(call_a16, 0xCD, 3, 24);
    (call_a16.handler)(&mut cpu, &mut mmu, [0x0, 0x50, 0x10, 0x0]); // call 0x1050
    //current pc should be 0x03, therefore
    //assert_eq!(mmu.read_byte(0xFFFE as usize), 0x03);
    //assert_eq!(mmu.read_byte(0xFFFF as usize), 0x03);
    assert_eq!(mmu.read_byte(0xFFFD as usize), 0x03);
    
    assert_eq!(cpu.sp, 0xFFFD);
    assert_eq!(cpu.pc, 0x1050);
    //check stack some more

    
    
    g!(call_ccz_a16, 0xCC, 3, 24);
    cpu.pc = 0;
    cpu.reset_flag(Flag::Z);
    assert_eq!(cpu.get_flag(Flag::Z), false);
    (call_ccz_a16.handler)(&mut cpu, &mut mmu, [0x0, 0x22, 0x22, 0x0]);
    assert_eq!(cpu.pc, 0x03);
    cpu.set_flag(Flag::Z);
    (call_ccz_a16.handler)(&mut cpu, &mut mmu, [0x0, 0x22, 0x22, 0x0]);
    assert_eq!(cpu.pc, 0x2222);
    assert_eq!(cpu.sp, 0xFFFB);

    g!(call_ccc_a16, 0xDC, 3, 24); //
    cpu.pc = 0x100;
    cpu.reset_flag(Flag::C);
    assert_eq!(cpu.get_flag(Flag::C), false);
    (call_ccc_a16.handler)(&mut cpu, &mut mmu, [0x0, 0x22, 0x22, 0x0]);
    assert_eq!(cpu.pc, 0x103);
    cpu.set_flag(Flag::C);
    (call_ccc_a16.handler)(&mut cpu, &mut mmu, [0x0, 0x22, 0x22, 0x0]);
    assert_eq!(cpu.pc, 0x2222);
    assert_eq!(cpu.sp, 0xFFF9);

    g!(ret, 0xC9, 1, 16);
    (ret.handler)(&mut cpu, &mut mmu, [0x0, 0x22, 0x22, 0x0]);
    assert_eq!(cpu.sp, 0xFFFB);
    assert_eq!(cpu.pc, 0x106);

}


#[test]
fn flag_test() {
    let (mut cpu, mut mmu) = prerequisites();
    cpu.f = 0b1000_0000;
    cpu.assign_flag(Flag::Z, true);
    assert_eq!(cpu.f, 0b1000_0000);
    cpu.assign_flag(Flag::Z, false);
    assert_eq!(cpu.f, 0);
    cpu.assign_flag(Flag::N, true);
    assert_eq!(cpu.f, 0b0100_0000);
    cpu.assign_flag(Flag::N, false);
    assert_eq!(cpu.f, 0);
}

fn to_bcd(val: u8) -> u8 {
    let msb = val / 10;
    let lsb = val % 10;
    msb << 4 | lsb
}