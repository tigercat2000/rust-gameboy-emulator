use std::{cell::RefCell, io::Read, sync::Mutex};

use tracing::{event, Level};

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
struct LCD {
    /// LCDC
    pub lcd_control: u8,
    /// SCY
    pub scroll_y: u8,
    /// LY
    pub lcd_y: u8,
    /// LYC
    pub lcd_y_cmp: u8,
}

impl Default for LCD {
    fn default() -> Self {
        Self {
            lcd_control: 0b1000_0000,
            scroll_y: 0,
            lcd_y: 0,
            lcd_y_cmp: 0,
        }
    }
}

#[derive(Debug)]
pub struct MemoryBus {
    program: Vec<u8>,
    vram: Mutex<RefCell<[u8; 0x1FFF]>>,
    lcd: Mutex<RefCell<LCD>>,
}

impl MemoryBus {
    pub fn new<R: Read>(mut reader: R) -> Self {
        let mut vec = Vec::new();
        reader.read_to_end(&mut vec).unwrap();
        Self {
            program: vec,
            vram: Mutex::new(RefCell::new([0; 0x1FFF])),
            lcd: Mutex::new(RefCell::new(LCD::default())),
        }
    }

    pub fn get_u8(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.program[addr as usize],
            0x8000..=0x9FFF => {
                let vram_guard = self.vram.try_lock();
                match vram_guard {
                    Ok(vram) => match vram.try_borrow() {
                        Ok(vram) => vram[addr as usize - 0x8000],
                        Err(_) => 0xFF,
                    },
                    Err(_) => 0xFF,
                }
            }
            0xFF40..=0xFF4B => {
                event!(Level::INFO, "LCD register read @{:#X}", addr);
                let lcd_guard = match self.lcd.try_lock() {
                    Ok(lcd_guard) => lcd_guard,
                    Err(_) => return 0xFF,
                };
                let lcd = match lcd_guard.try_borrow() {
                    Ok(lcd) => lcd,
                    Err(_) => return 0xFF,
                };

                match addr {
                    0xFF40 => lcd.lcd_control,
                    0xFF42 => lcd.scroll_y,
                    0xFF44 => lcd.lcd_y,
                    0xFF45 => lcd.lcd_y_cmp,
                    _ => unimplemented!(),
                }
            }
            0xFF00..=0xFF7F => {
                event!(Level::INFO, "IO register read @{:#X}", addr);
                unimplemented!()
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
            // VRAM!
            0x8000..=0x9FFF => {
                event!(
                    Level::INFO,
                    "VRAM write @{:#X}: {:#X} '{}'",
                    addr,
                    byte,
                    byte as char
                );
                let vram_guard = match self.vram.try_lock() {
                    Ok(vram_guard) => vram_guard,
                    Err(_) => return,
                };
                let mut vram = match vram_guard.try_borrow_mut() {
                    Ok(vram) => vram,
                    Err(_) => return,
                };
                vram[addr as usize - 0x8000] = byte
            }
            // LCD
            0xFF40..=0xFF4B => {
                event!(Level::INFO, "LCD register write @{:#X}: {:#X}", addr, byte);
                let lcd_guard = match self.lcd.try_lock() {
                    Ok(lcd_guard) => lcd_guard,
                    Err(_) => return,
                };
                let mut lcd = match lcd_guard.try_borrow_mut() {
                    Ok(lcd) => lcd,
                    Err(_) => return,
                };
                match addr {
                    0xFF40 => lcd.lcd_control = byte,
                    0xFF42 => lcd.scroll_y = byte,
                    0xFF44 => lcd.lcd_y = byte,
                    0xFF45 => lcd.lcd_y_cmp = byte,
                    _ => {}
                }
            }
            // I/O registers
            0xFF00..=0xFF7F => {
                event!(Level::INFO, "IO register write @{:#X}: {:#X}", addr, byte);
            }
            _ => panic!("Illegal memory write at {:#X}", addr),
        }
    }
}
