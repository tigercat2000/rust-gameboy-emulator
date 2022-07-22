macro_rules! test_success {
    ($name:ident, $input:expr => $output:expr) => {
        #[test]
        fn $name() {
            let (rest, instr) = Instruction::parse(&$input)
                .expect("Instruction parse failed when it shouldn't have");
            assert_eq!(rest.len(), 0);
            assert_eq!(instr, $output);
        }
    };
}

pub mod alu;
pub mod instructions;
