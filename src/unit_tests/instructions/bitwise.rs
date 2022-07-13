use super::*;

macro_rules! bitwise_cases {
    ($name:ident, $base:expr, $outer:expr, $enum:expr) => {
        paste! {
            test_success!([<$name _ b>], [0xCB, $base + 0x0] => $outer($enum, Register8::B));
            test_success!([<$name _ c>], [0xCB, $base + 0x1] => $outer($enum, Register8::C));
            test_success!([<$name _ d>], [0xCB, $base + 0x2] => $outer($enum, Register8::D));
            test_success!([<$name _ e>], [0xCB, $base + 0x3] => $outer($enum, Register8::E));
            test_success!([<$name _ h>], [0xCB, $base + 0x4] => $outer($enum, Register8::H));
            test_success!([<$name _ l>], [0xCB, $base + 0x5] => $outer($enum, Register8::L));
            test_success!([<$name _ indirect>], [0xCB, $base + 0x6] => $outer($enum, Register8::IndirectHL));
            test_success!([<$name _ a>], [0xCB, $base + 0x7] => $outer($enum, Register8::A));
        }
    };
}

bitwise_cases!(
    rotate_left_carry,
    0x00,
    Instruction::Bitwise,
    BitwiseOp::RotateLeftCarry
);
bitwise_cases!(
    rotate_right_carry,
    0x08,
    Instruction::Bitwise,
    BitwiseOp::RotateRightCarry
);
bitwise_cases!(
    rotate_left,
    0x10,
    Instruction::Bitwise,
    BitwiseOp::RotateLeft
);
bitwise_cases!(
    rotate_right,
    0x18,
    Instruction::Bitwise,
    BitwiseOp::RotateRight
);
bitwise_cases!(
    shift_left_arithmetic,
    0x20,
    Instruction::Bitwise,
    BitwiseOp::ShiftLeftArithmetic
);
bitwise_cases!(
    shift_right_arithmetic,
    0x28,
    Instruction::Bitwise,
    BitwiseOp::ShiftRightArithmetic
);
bitwise_cases!(swap, 0x30, Instruction::Bitwise, BitwiseOp::Swap);
bitwise_cases!(
    shift_right_logical,
    0x38,
    Instruction::Bitwise,
    BitwiseOp::ShiftRightLogical
);

bitwise_cases!(bit_zero, 0x40, Instruction::Bit, 0);
bitwise_cases!(bit_one, 0x48, Instruction::Bit, 1);
bitwise_cases!(bit_two, 0x50, Instruction::Bit, 2);
bitwise_cases!(bit_three, 0x58, Instruction::Bit, 3);
bitwise_cases!(bit_four, 0x60, Instruction::Bit, 4);
bitwise_cases!(bit_five, 0x68, Instruction::Bit, 5);
bitwise_cases!(bit_six, 0x70, Instruction::Bit, 6);
bitwise_cases!(bit_seven, 0x78, Instruction::Bit, 7);

bitwise_cases!(reset_bit_zero, 0x80, Instruction::ResetBit, 0);
bitwise_cases!(reset_bit_one, 0x88, Instruction::ResetBit, 1);
bitwise_cases!(reset_bit_two, 0x90, Instruction::ResetBit, 2);
bitwise_cases!(reset_bit_three, 0x98, Instruction::ResetBit, 3);
bitwise_cases!(reset_bit_four, 0xA0, Instruction::ResetBit, 4);
bitwise_cases!(reset_bit_five, 0xA8, Instruction::ResetBit, 5);
bitwise_cases!(reset_bit_six, 0xB0, Instruction::ResetBit, 6);
bitwise_cases!(reset_bit_seven, 0xB8, Instruction::ResetBit, 7);

bitwise_cases!(set_bit_zero, 0xC0, Instruction::SetBit, 0);
bitwise_cases!(set_bit_one, 0xC8, Instruction::SetBit, 1);
bitwise_cases!(set_bit_two, 0xD0, Instruction::SetBit, 2);
bitwise_cases!(set_bit_three, 0xD8, Instruction::SetBit, 3);
bitwise_cases!(set_bit_four, 0xE0, Instruction::SetBit, 4);
bitwise_cases!(set_bit_five, 0xE8, Instruction::SetBit, 5);
bitwise_cases!(set_bit_six, 0xF0, Instruction::SetBit, 6);
bitwise_cases!(set_bit_seven, 0xF8, Instruction::SetBit, 7);
