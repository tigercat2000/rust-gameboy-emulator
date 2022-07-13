use super::*;

macro_rules! alu_cases {
    ($name:ident, $base:expr, $enum:expr) => {
        paste! {
            test_success!([<$name _ b>], [$base + 0x0] => Instruction::Alu($enum, Register8::B));
            test_success!([<$name _ c>], [$base + 0x1] => Instruction::Alu($enum, Register8::C));
            test_success!([<$name _ d>], [$base + 0x2] => Instruction::Alu($enum, Register8::D));
            test_success!([<$name _ e>], [$base + 0x3] => Instruction::Alu($enum, Register8::E));
            test_success!([<$name _ h>], [$base + 0x4] => Instruction::Alu($enum, Register8::H));
            test_success!([<$name _ l>], [$base + 0x5] => Instruction::Alu($enum, Register8::L));
            test_success!([<$name _ indirect>], [$base + 0x6] => Instruction::Alu($enum, Register8::IndirectHL));
            test_success!([<$name _ a>], [$base + 0x7] => Instruction::Alu($enum, Register8::A));
        }
    };
}

alu_cases!(add, 0x80, AluOp::Add);
alu_cases!(adc, 0x88, AluOp::AddWithCarry);
alu_cases!(sub, 0x90, AluOp::Subtract);
alu_cases!(sbc, 0x98, AluOp::SubtractWithCarry);
alu_cases!(and, 0xA0, AluOp::And);
alu_cases!(xor, 0xA8, AluOp::Xor);
alu_cases!(or, 0xB0, AluOp::Or);
alu_cases!(compare, 0xB8, AluOp::Compare);

// Immediates
test_success!(add_a_immediate, [0xC6, 0x69] => Instruction::AluImmediate(AluOp::Add, 0x69));
test_success!(sub_a_immediate, [0xD6, 0x69] => Instruction::AluImmediate(AluOp::Subtract, 0x69));
test_success!(and_a_immediate, [0xE6, 0x69] => Instruction::AluImmediate(AluOp::And, 0x69));
test_success!(or_a_immediate, [0xF6, 0x69] => Instruction::AluImmediate(AluOp::Or, 0x69));
test_success!(adc_a_immediate, [0xCE, 0x69] => Instruction::AluImmediate(AluOp::AddWithCarry, 0x69));
test_success!(sbc_a_immediate, [0xDE, 0x69] => Instruction::AluImmediate(AluOp::SubtractWithCarry, 0x69));
test_success!(xor_a_immediate, [0xEE, 0x69] => Instruction::AluImmediate(AluOp::Xor, 0x69));
test_success!(cp_a_immediate, [0xFE, 0x69] => Instruction::AluImmediate(AluOp::Compare, 0x69));
