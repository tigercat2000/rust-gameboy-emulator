#![allow(clippy::bool_assert_comparison)]
use crate::emulator::cpu::{Flag, ALU, CPU};

#[test]
fn test_sra() {
    let value = 0b10100111;
    let expected = 0b11010011;
    let mut dummy = CPU::default();
    let actual = ALU::shift_right_arithmetic(&mut dummy, value);
    assert_eq!(actual, expected);
    assert_eq!(dummy.get_flag(Flag::C), true);
}

#[test]
fn test_sla() {
    let value = 0b10100111;
    let expected = 0b01001110;
    let mut dummy = CPU::default();
    let actual = ALU::shift_left_arithmetic(&mut dummy, value);
    assert_eq!(actual, expected);
    assert_eq!(dummy.get_flag(Flag::C), true);
}

#[test]
fn test_srl() {
    let value = 0b10100111;
    let expected = 0b01010011;
    let mut dummy = CPU::default();
    let actual = ALU::shift_right_logical(&mut dummy, value);
    assert_eq!(actual, expected);
    assert_eq!(dummy.get_flag(Flag::C), true);
}
