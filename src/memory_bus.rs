use std::io::Read;

#[derive(Debug)]
pub struct MemoryBus {
    program: Vec<u8>,
}

impl MemoryBus {
    pub fn new<R: Read>(mut reader: R) -> Self {
        let mut vec = Vec::new();
        reader.read_to_end(&mut vec).unwrap();
        Self { program: vec }
    }

    pub fn get_u8(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.program[addr as usize],
            0xFF00..=0xFF7F => {
                println!("IO register read: {:#X}", addr);
                0
            }
            _ => unimplemented!(),
        }
    }

    #[allow(clippy::identity_op)]
    pub fn get_instr(&self, addr: u16) -> [u8; 4] {
        [
            self.program[(addr + 0) as usize],
            self.program[(addr + 1) as usize],
            self.program[(addr + 2) as usize],
            self.program[(addr + 3) as usize],
        ]
    }

    pub fn write_u8(&self, addr: u16, byte: u8) {
        match addr {
            0xFF00..=0xFF7F => {
                println!("IO register write: {:#X} -> {:#X}", byte, addr);
            }
            _ => panic!("Illegal memory write at {:#X}", addr),
        }
    }
}
