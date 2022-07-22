#![allow(non_snake_case)]
use crate::emulator::instructions::{
    AccumulatorFlagOp, AluOp, BitwiseOp, Condition, Instruction, Register16, Register16Indirect,
    Register16Stack, Register8,
};

use paste::paste;

pub mod alu;
pub mod bitwise;
pub mod load;

test_success!(nop, [0x00] => Instruction::Nop);
test_success!(load_sp, [0x08, 0xAD, 0xDE] => Instruction::LoadSP(0xDEAD));
test_success!(stop, [0x10] => Instruction::Stop);
// JR
test_success!(jr_unconditional, [0x18, 0xA] => Instruction::JumpRelative(0xA));
test_success!(jr_Z, [0x28, 0xA] => Instruction::JumpRelativeConditional(Condition::Z, 0xA));
test_success!(jr_C, [0x38, 0xA] => Instruction::JumpRelativeConditional(Condition::C, 0xA));
test_success!(jr_NZ, [0x20, 0xA] => Instruction::JumpRelativeConditional(Condition::NZ, 0xA));
test_success!(jr_NC, [0x30, 0xA] => Instruction::JumpRelativeConditional(Condition::NC, 0xA));
// Add HL
test_success!(add_hl_bc, [0x09] => Instruction::AddHLRegister(Register16::BC));
test_success!(add_hl_de, [0x19] => Instruction::AddHLRegister(Register16::DE));
test_success!(add_hl_hl, [0x29] => Instruction::AddHLRegister(Register16::HL));
test_success!(add_hl_sp, [0x39] => Instruction::AddHLRegister(Register16::SP));
// Increment 16
test_success!(increment_bc, [0x03] => Instruction::Increment16(Register16::BC));
test_success!(increment_de, [0x13] => Instruction::Increment16(Register16::DE));
test_success!(increment_hl, [0x23] => Instruction::Increment16(Register16::HL));
test_success!(increment_sp, [0x33] => Instruction::Increment16(Register16::SP));
// Decrement 16
test_success!(decrement_bc, [0x0B] => Instruction::Decrement16(Register16::BC));
test_success!(decrement_de, [0x1B] => Instruction::Decrement16(Register16::DE));
test_success!(decrement_hl, [0x2B] => Instruction::Decrement16(Register16::HL));
test_success!(decrement_sp, [0x3B] => Instruction::Decrement16(Register16::SP));
// Increment 8
test_success!(increment_b, [0x04] => Instruction::Increment(Register8::B));
test_success!(increment_d, [0x14] => Instruction::Increment(Register8::D));
test_success!(increment_h, [0x24] => Instruction::Increment(Register8::H));
test_success!(increment_indirect, [0x34] => Instruction::Increment(Register8::IndirectHL));
test_success!(increment_c, [0x0C] => Instruction::Increment(Register8::C));
test_success!(increment_e, [0x1C] => Instruction::Increment(Register8::E));
test_success!(increment_l, [0x2C] => Instruction::Increment(Register8::L));
test_success!(increment_a, [0x3C] => Instruction::Increment(Register8::A));
// Decrement 8
test_success!(decrement_b, [0x05] => Instruction::Decrement(Register8::B));
test_success!(decrement_d, [0x15] => Instruction::Decrement(Register8::D));
test_success!(decrement_h, [0x25] => Instruction::Decrement(Register8::H));
test_success!(decrement_indirect, [0x35] => Instruction::Decrement(Register8::IndirectHL));
test_success!(decrement_c, [0x0D] => Instruction::Decrement(Register8::C));
test_success!(decrement_e, [0x1D] => Instruction::Decrement(Register8::E));
test_success!(decrement_l, [0x2D] => Instruction::Decrement(Register8::L));
test_success!(decrement_a, [0x3D] => Instruction::Decrement(Register8::A));
// Load Immediate 8
test_success!(load_b_imm, [0x06, 0x69] => Instruction::LoadImmediate(Register8::B, 0x69));
test_success!(load_d_imm, [0x16, 0x69] => Instruction::LoadImmediate(Register8::D, 0x69));
test_success!(load_h_imm, [0x26, 0x69] => Instruction::LoadImmediate(Register8::H, 0x69));
test_success!(load_indirect_imm, [0x36, 0x69] => Instruction::LoadImmediate(Register8::IndirectHL, 0x69));

test_success!(load_c_imm, [0x0E, 0x69] => Instruction::LoadImmediate(Register8::C, 0x69));
test_success!(load_e_imm, [0x1E, 0x69] => Instruction::LoadImmediate(Register8::E, 0x69));
test_success!(load_l_imm, [0x2E, 0x69] => Instruction::LoadImmediate(Register8::L, 0x69));
test_success!(load_a_imm, [0x3E, 0x69] => Instruction::LoadImmediate(Register8::A, 0x69));
// ACC ops
test_success!(rotate_left_carry_a, [0x07] => Instruction::AccumulatorFlag(AccumulatorFlagOp::RotateLeftCarryA));
test_success!(rotate_left_a, [0x17] => Instruction::AccumulatorFlag(AccumulatorFlagOp::RotateLeftA));
test_success!(decimal_adjust_after_addition, [0x27] => Instruction::AccumulatorFlag(AccumulatorFlagOp::DecimalAdjustAfterAddition));
test_success!(set_carry_flag, [0x37] => Instruction::AccumulatorFlag(AccumulatorFlagOp::SetCarryFlag));
test_success!(rotate_right_carry_a, [0x0F] => Instruction::AccumulatorFlag(AccumulatorFlagOp::RotateRightCarryA));
test_success!(rotate_right_a, [0x1F] => Instruction::AccumulatorFlag(AccumulatorFlagOp::RotateRightA));
test_success!(complement_accumulator, [0x2F] => Instruction::AccumulatorFlag(AccumulatorFlagOp::ComplementAccumulator));
test_success!(complement_carry_flag, [0x3F] => Instruction::AccumulatorFlag(AccumulatorFlagOp::ComplementCarryFlag));
// Group 2
test_success!(halt, [0x76] => Instruction::Halt);
// Ret
test_success!(ret_NZ, [0xC0] => Instruction::RetConditional(Condition::NZ));
test_success!(ret_Z, [0xC8] => Instruction::RetConditional(Condition::Z));
test_success!(ret_NC, [0xD0] => Instruction::RetConditional(Condition::NC));
test_success!(ret_C, [0xD8] => Instruction::RetConditional(Condition::C));
// LDH (n),A / LD (0xFF00 + u8) (n),A
test_success!(load_highpage_a, [0xE0, 0x69] => Instruction::LoadHighPageA(0x69));
// ADD SP, i8
test_success!(add_sp, [0xE8, 0x5] => Instruction::AddSp(0x5));
// LDH A,(n) / LD (0xFF00 + u8) A,(n)
test_success!(load_a_highpage, [0xF0, 0x69] => Instruction::LoadAHighPage(0x69));
test_success!(load_hl_sp, [0xF8, 0x5] => Instruction::LoadHLSP(0x5));
// Pop
test_success!(pop_bc, [0xC1] => Instruction::Pop(Register16Stack::BC));
test_success!(pop_de, [0xD1] => Instruction::Pop(Register16Stack::DE));
test_success!(pop_hl, [0xE1] => Instruction::Pop(Register16Stack::HL));
test_success!(pop_af, [0xF1] => Instruction::Pop(Register16Stack::AF));
// Ret
test_success!(ret, [0xC9] => Instruction::Ret);
test_success!(ret_interrupt, [0xD9] => Instruction::RetInterrupt);
test_success!(jump_hl, [0xE9] => Instruction::JumpHL);
test_success!(load_sp_hl, [0xF9] => Instruction::LoadSPHL);
// JP condition (u16)
test_success!(jump_NZ, [0xC2, 0xAD, 0xDE] => Instruction::JumpConditional(Condition::NZ, 0xDEAD));
test_success!(jump_Z, [0xCA, 0xAD, 0xDE] => Instruction::JumpConditional(Condition::Z, 0xDEAD));
test_success!(jump_NC, [0xD2, 0xAD, 0xDE] => Instruction::JumpConditional(Condition::NC, 0xDEAD));
test_success!(jump_C, [0xDA, 0xAD, 0xDE] => Instruction::JumpConditional(Condition::C, 0xDEAD));
// LD (FF00+C), A/LD (C),A
test_success!(load_highpage_indirect_a, [0xE2] => Instruction::LoadHighPageIndirectA);
// LD (u16),A
test_success!(load_indirect_immediate_a, [0xEA, 0xAD, 0xDE] => Instruction::LoadIndirectImmediateA(0xDEAD));
// LD A,(FF00+C)/LD A,(C)
test_success!(load_highpage_a_indirect, [0xF2] => Instruction::LoadAHighPageIndirect);
// LD A,(u16)
test_success!(load_a_indirect_immediate, [0xFA, 0xAD, 0xDE] => Instruction::LoadAIndirectImmediate(0xDEAD));
test_success!(jump_unconditional, [0xC3, 0xAD, 0xDE] => Instruction::Jump(0xDEAD));
test_success!(disable_interrupts, [0xF3] => Instruction::DisableInterrupts);
test_success!(enable_interrupts, [0xFB] => Instruction::EnableInterrupts);
// Call
test_success!(call_NZ, [0xC4, 0xAD, 0xDE] => Instruction::CallConditional(Condition::NZ, 0xDEAD));
test_success!(call_Z, [0xCC, 0xAD, 0xDE] => Instruction::CallConditional(Condition::Z, 0xDEAD));
test_success!(call_NC, [0xD4, 0xAD, 0xDE] => Instruction::CallConditional(Condition::NC, 0xDEAD));
test_success!(call_C, [0xDC, 0xAD, 0xDE] => Instruction::CallConditional(Condition::C, 0xDEAD));
test_success!(call, [0xCD, 0xAD, 0xDE] => Instruction::Call(0xDEAD));
test_success!(push_bc, [0xC5] => Instruction::Push(Register16Stack::BC));
test_success!(push_de, [0xD5] => Instruction::Push(Register16Stack::DE));
test_success!(push_hl, [0xE5] => Instruction::Push(Register16Stack::HL));
test_success!(push_af, [0xF5] => Instruction::Push(Register16Stack::AF));
test_success!(reset00, [0xC7] => Instruction::Reset(0b000));
test_success!(reset08, [0xCF] => Instruction::Reset(0b001));
test_success!(reset10, [0xD7] => Instruction::Reset(0b010));
test_success!(reset18, [0xDF] => Instruction::Reset(0b011));
test_success!(reset20, [0xE7] => Instruction::Reset(0b100));
test_success!(reset28, [0xEF] => Instruction::Reset(0b101));
test_success!(reset30, [0xF7] => Instruction::Reset(0b110));
test_success!(reset38, [0xFF] => Instruction::Reset(0b111));
