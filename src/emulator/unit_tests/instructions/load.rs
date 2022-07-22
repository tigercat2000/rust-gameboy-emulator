use super::*;

mod sixteen {
    use super::*;
    // LD r16, u16
    test_success!(load_bc_imm, [0x01, 0xAD, 0xDE] => Instruction::LoadImmediate16(Register16::BC, 0xDEAD));
    test_success!(load_de_imm, [0x11, 0xAD, 0xDE] => Instruction::LoadImmediate16(Register16::DE, 0xDEAD));
    test_success!(load_hl_imm, [0x21, 0xAD, 0xDE] => Instruction::LoadImmediate16(Register16::HL, 0xDEAD));
    test_success!(load_sp_imm, [0x31, 0xAD, 0xDE] => Instruction::LoadImmediate16(Register16::SP, 0xDEAD));
    // Load indirect A
    test_success!(load_indirect_a_bc, [0x02] => Instruction::LoadIndirectA(Register16Indirect::BC));
    test_success!(load_indirect_a_de, [0x12] => Instruction::LoadIndirectA(Register16Indirect::DE));
    test_success!(load_indirect_a_hli, [0x22] => Instruction::LoadIndirectA(Register16Indirect::HLI));
    test_success!(load_indirect_a_hld, [0x32] => Instruction::LoadIndirectA(Register16Indirect::HLD));
    // Load A indirect
    test_success!(load_a_bc_indirect, [0x0A] => Instruction::LoadAIndirect(Register16Indirect::BC));
    test_success!(load_a_de_indirect, [0x1A] => Instruction::LoadAIndirect(Register16Indirect::DE));
    test_success!(load_a_hli_indirect, [0x2A] => Instruction::LoadAIndirect(Register16Indirect::HLI));
    test_success!(load_a_hld_indirect, [0x3A] => Instruction::LoadAIndirect(Register16Indirect::HLD));
}

mod eight {
    use super::*;

    macro_rules! load_cases {
        ($name:ident, $base:expr, $outer:expr, $operand:expr) => {
            paste! {
                test_success!([<$name _b>], [$base + 0x0] => $outer($operand, Register8::B));
                test_success!([<$name _c>], [$base + 0x1] => $outer($operand, Register8::C));
                test_success!([<$name _d>], [$base + 0x2] => $outer($operand, Register8::D));
                test_success!([<$name _e>], [$base + 0x3] => $outer($operand, Register8::E));
                test_success!([<$name _h>], [$base + 0x4] => $outer($operand, Register8::H));
                test_success!([<$name _l>], [$base + 0x5] => $outer($operand, Register8::L));
                test_success!([<$name _indirect>], [$base + 0x6] => $outer($operand, Register8::IndirectHL));
                test_success!([<$name _a>], [$base + 0x7] => $outer($operand, Register8::A));
            }
        }
    }

    /// Load cases except indirect
    /// this isn't really less code its just less effort
    macro_rules! load_cases_noindirect {
        ($name:ident, $base:expr, $outer:expr, $operand:expr) => {
            paste! {
                test_success!([<$name _b>], [$base + 0x0] => $outer($operand, Register8::B));
                test_success!([<$name _c>], [$base + 0x1] => $outer($operand, Register8::C));
                test_success!([<$name _d>], [$base + 0x2] => $outer($operand, Register8::D));
                test_success!([<$name _e>], [$base + 0x3] => $outer($operand, Register8::E));
                test_success!([<$name _h>], [$base + 0x4] => $outer($operand, Register8::H));
                test_success!([<$name _l>], [$base + 0x5] => $outer($operand, Register8::L));
                test_success!([<$name _a>], [$base + 0x7] => $outer($operand, Register8::A));
            }
        };
    }

    load_cases!(load_b, 0x40, Instruction::Load, Register8::B);
    load_cases!(load_c, 0x48, Instruction::Load, Register8::C);
    load_cases!(load_d, 0x50, Instruction::Load, Register8::D);
    load_cases!(load_e, 0x58, Instruction::Load, Register8::E);
    load_cases!(load_h, 0x60, Instruction::Load, Register8::H);
    load_cases!(load_l, 0x68, Instruction::Load, Register8::L);
    // load_cases!(
    //     load_indirect,
    //     0x70,
    //     Instruction::Load,
    //     Register8::IndirectHL
    // );
    // Load (HL), (HL) is actually HALT :)
    load_cases_noindirect!(load_hl, 0x70, Instruction::Load, Register8::IndirectHL);
    load_cases!(load_a, 0x78, Instruction::Load, Register8::A);
}
