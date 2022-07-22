use std::{cell::RefCell, io::Read, sync::Mutex};

use bit_field::BitField;
use tracing::{trace, warn};

pub const LCDC: u16 = 0xFF40;
pub const SCROLL_Y: u16 = 0xFF42;
pub const SCROLL_X: u16 = 0xFF43;
pub const LCD_Y: u16 = 0xFF44;
pub const PALLETE: u16 = 0xFF47;
pub const IF: u16 = 0xFF0F;
pub const IE: u16 = 0xFFFF;

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
struct LCD {
    /// LCDC
    pub lcd_control: u8,
    /// SCY
    pub scroll_y: u8,
    /// SCX
    pub scroll_x: u8,
    /// LY
    pub lcd_y: u8,
    /// LYC
    pub lcd_y_cmp: u8,
    /// BGP
    pub background_pallete: u8,
    /// WY
    pub window_y: u8,
    /// WX
    pub window_x: u8,
}

impl Default for LCD {
    fn default() -> Self {
        Self {
            lcd_control: 0b1000_0000,
            scroll_y: 0,
            scroll_x: 0,
            lcd_y: 0,
            lcd_y_cmp: 0,
            background_pallete: 0,
            window_y: 0,
            window_x: 0,
        }
    }
}

pub enum Interrupt {
    /// INT 40
    VBlank,
    /// INT 48
    LCDStat,
    /// INT 50
    Timer,
    /// INT 58
    Serial,
    /// INT 60
    Joypad,
}

#[derive(Debug, Default)]
struct Interrupts {
    /// INT 40
    pub vblank_enabled: bool,
    /// INT 48
    pub lcd_stat_enabled: bool,
    /// INT 50
    pub timer_enabled: bool,
    /// INT 58
    pub serial_enabled: bool,
    /// INT 60
    pub joypad_enabled: bool,

    /// INT 40
    pub vblank_requested: bool,
    /// INT 48
    pub lcd_stat_requested: bool,
    /// INT 50
    pub timer_requested: bool,
    /// INT 58
    pub serial_requested: bool,
    /// INT 60
    pub joypad_requested: bool,
}

impl Interrupts {
    fn get_interrupt_enable(&self) -> u8 {
        let mut new_number = 0;
        new_number.set_bit(0, self.vblank_enabled);
        new_number.set_bit(1, self.lcd_stat_enabled);
        new_number.set_bit(2, self.timer_enabled);
        new_number.set_bit(3, self.serial_enabled);
        new_number.set_bit(4, self.joypad_enabled);
        new_number
    }

    fn set_interrupt_enable(&mut self, byte: u8) {
        self.vblank_enabled = byte.get_bit(0);
        self.lcd_stat_enabled = byte.get_bit(1);
        self.timer_enabled = byte.get_bit(2);
        self.serial_enabled = byte.get_bit(3);
        self.joypad_enabled = byte.get_bit(4);
    }

    fn get_interrupt_flag(&self) -> u8 {
        let mut new_number = 0;
        new_number.set_bit(0, self.vblank_requested);
        new_number.set_bit(1, self.lcd_stat_requested);
        new_number.set_bit(2, self.timer_requested);
        new_number.set_bit(3, self.serial_requested);
        new_number.set_bit(4, self.joypad_requested);
        new_number
    }

    fn set_interrupt_flag(&mut self, byte: u8) {
        self.vblank_requested = byte.get_bit(0);
        self.lcd_stat_requested = byte.get_bit(1);
        self.timer_requested = byte.get_bit(2);
        self.serial_requested = byte.get_bit(3);
        self.joypad_requested = byte.get_bit(4);
    }
}

#[derive(Debug)]
pub struct MemoryBus {
    program: Vec<u8>,
    wram1: Mutex<RefCell<[u8; 0xCFFF - 0xC000 + 1]>>,
    wram2: Mutex<RefCell<[u8; 0xDFFF - 0xD000 + 1]>>,
    vram: Mutex<RefCell<[u8; 0x1FFF + 1]>>,
    oam: Mutex<RefCell<[u8; 0xFE9F - 0xFE00 + 1]>>,
    hram: Mutex<RefCell<[u8; 0xFFFE - 0xFF80 + 1]>>,
    lcd: Mutex<RefCell<LCD>>,
    interrupts: Mutex<RefCell<Interrupts>>,
}

impl MemoryBus {
    pub fn new<R: Read>(mut reader: R) -> Self {
        let mut vec = Vec::new();
        reader.read_to_end(&mut vec).unwrap();
        Self {
            program: vec,
            wram1: Mutex::new(RefCell::new([0; 0xCFFF - 0xC000 + 1])),
            wram2: Mutex::new(RefCell::new([0; 0xDFFF - 0xD000 + 1])),
            vram: Mutex::new(RefCell::new([0; 0x1FFF + 1])),
            oam: Mutex::new(RefCell::new([0; 0xFE9F - 0xFE00 + 1])),
            hram: Mutex::new(RefCell::new([0; 0xFFFE - 0xFF80 + 1])),
            lcd: Mutex::new(RefCell::new(LCD::default())),
            interrupts: Mutex::new(RefCell::new(Interrupts::default())),
        }
    }

    pub fn get_stack_16(&self, sp: &mut u16) -> u16 {
        let upper = self.get_stack(sp) as u16;
        let lower = self.get_stack(sp) as u16;
        upper << 8 | lower
    }

    pub fn get_stack(&self, sp: &mut u16) -> u8 {
        let val = self.get_u8(*sp);
        *sp = sp.wrapping_add(1);
        val
    }

    pub fn get_u8(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => {
                trace!("PROG read @{:#X}", addr);
                self.program[addr as usize]
            }
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
            // WRAM 1
            0xC000..=0xCFFF => {
                let wram_guard = self.wram1.try_lock();
                let val = match wram_guard {
                    Ok(wram) => match wram.try_borrow() {
                        Ok(wram) => wram[addr as usize - 0xC000],
                        Err(_) => 0xFF,
                    },
                    Err(_) => 0xFF,
                };
                trace!("WRAM read @{:#X}: {:#X}", addr, val);
                val
            }
            // WRAM 1
            0xD000..=0xDFFF => {
                let wram_guard = self.wram2.try_lock();
                let val = match wram_guard {
                    Ok(wram) => match wram.try_borrow() {
                        Ok(wram) => wram[addr as usize - 0xD000],
                        Err(_) => 0xFF,
                    },
                    Err(_) => 0xFF,
                };
                trace!("WRAM read @{:#X}: {:#X}", addr, val);
                val
            }
            // OAM
            0xFE00..=0xFE9F => {
                let oam_guard = self.oam.try_lock();
                let val = match oam_guard {
                    Ok(oam) => match oam.try_borrow() {
                        Ok(oam) => oam[addr as usize - 0xFE00],
                        Err(_) => 0xFF,
                    },
                    Err(_) => 0xFF,
                };
                trace!("OAM read @{:#X}: {:#X}", addr, val);
                val
            }
            0xFF40..=0xFF4B => {
                trace!("LCD register read @{:#X}", addr);
                let lcd_guard = match self.lcd.try_lock() {
                    Ok(lcd_guard) => lcd_guard,
                    Err(_) => return 0xFF,
                };
                let lcd = match lcd_guard.try_borrow() {
                    Ok(lcd) => lcd,
                    Err(_) => return 0xFF,
                };

                match addr {
                    LCDC => lcd.lcd_control,
                    SCROLL_Y => lcd.scroll_y,
                    SCROLL_X => lcd.scroll_x,
                    LCD_Y => lcd.lcd_y,
                    0xFF45 => lcd.lcd_y_cmp,
                    PALLETE => lcd.background_pallete,
                    0xFF4A => lcd.window_y,
                    0xFF4B => lcd.window_x,
                    _ => unimplemented!(),
                }
            }
            // Interrupt Flag (IF)
            IF => {
                let interrupts_guard = self.interrupts.try_lock();
                let val = match interrupts_guard {
                    Ok(interrupts) => match interrupts.try_borrow() {
                        Ok(interrupts) => interrupts.get_interrupt_flag(),
                        Err(_) => return 0x00,
                    },
                    Err(_) => return 0x00,
                };
                trace!("IF read @{:#X}: {:#X}", addr, val);
                val
            }
            // Interrupt Enable (IE)
            IE => {
                let interrupts_guard = self.interrupts.try_lock();
                let val = match interrupts_guard {
                    Ok(interrupts) => match interrupts.try_borrow() {
                        Ok(interrupts) => interrupts.get_interrupt_enable(),
                        Err(_) => return 0x00,
                    },
                    Err(_) => return 0x00,
                };
                trace!("IE read @{:#X}: {:#X}", addr, val);
                val
            }
            0xFF00..=0xFF7F => {
                warn!("Unimplemented IO register read @{:#X}", addr);
                0x00
            }
            0xFF80..=0xFFFE => {
                let hram_guard = self.hram.try_lock();
                let val = match hram_guard {
                    Ok(hram) => match hram.try_borrow() {
                        Ok(hram) => hram[addr as usize - 0xFF80],
                        Err(_) => 0xFF,
                    },
                    Err(_) => 0xFF,
                };
                trace!("HRAM read @{:#X}: {:#X}", addr, val);
                val
            }
            _ => unimplemented!(),
        }
    }

    #[allow(clippy::identity_op)]
    pub fn get_instr(&self, addr: u16) -> [u8; 4] {
        if addr >= 0x7FFF {
            warn!("Reading instruction outside of ROM @{:#X}", addr);
        }
        [
            self.get_u8(addr + 0),
            self.get_u8(addr + 1),
            self.get_u8(addr + 2),
            self.get_u8(addr + 3),
        ]
    }

    // TODO: Find out if this is the correct order
    pub fn write_stack_16(&self, sp: &mut u16, word: u16) {
        self.write_stack(sp, word.get_bits(0..8) as u8);
        self.write_stack(sp, word.get_bits(8..16) as u8);
    }

    pub fn write_stack(&self, sp: &mut u16, byte: u8) {
        *sp = sp.wrapping_sub(1);
        self.write_u8(*sp, byte);
    }

    pub fn write_u8(&self, addr: u16, byte: u8) {
        match addr {
            0x0000..=0x7FFF => {
                warn!(
                    "(continuing) Illegal write to ROM @{:#X}: {:#X}",
                    addr, byte
                );
                // Allow it anyways
            }
            // VRAM!
            0x8000..=0x9FFF => {
                trace!("VRAM write @{:#X}: {:#X} '{}'", addr, byte, byte as char);
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
            // WRAM 1
            0xC000..=0xCFFF => {
                trace!("WRAM write @{:#X}: {:#X}", addr, byte);
                let wram_guard = match self.wram1.try_lock() {
                    Ok(wram_guard) => wram_guard,
                    Err(_) => return,
                };
                let mut wram = match wram_guard.try_borrow_mut() {
                    Ok(wram) => wram,
                    Err(_) => return,
                };
                wram[addr as usize - 0xC000] = byte
            }
            // WRAM 2
            0xD000..=0xDFFF => {
                trace!("WRAM write @{:#X}: {:#X}", addr, byte);
                let wram_guard = match self.wram2.try_lock() {
                    Ok(wram_guard) => wram_guard,
                    Err(_) => return,
                };
                let mut wram = match wram_guard.try_borrow_mut() {
                    Ok(wram) => wram,
                    Err(_) => return,
                };
                wram[addr as usize - 0xD000] = byte
            }
            // OAM
            0xFE00..=0xFE9F => {
                trace!("OAM write @{:#X}: {:#X}", addr, byte);
                let oam_guard = match self.oam.try_lock() {
                    Ok(oam_guard) => oam_guard,
                    Err(_) => return,
                };
                let mut oam = match oam_guard.try_borrow_mut() {
                    Ok(oam) => oam,
                    Err(_) => return,
                };
                oam[addr as usize - 0xFE00] = byte
            }
            0xFEA0..=0xFEFF => {
                warn!(
                    "(continuing) Illegal write to prohibited zone @{:#X}: {:#X}",
                    addr, byte
                );
            }
            // LCD
            0xFF40..=0xFF4B => {
                trace!("LCD register write @{:#X}: {:#X}", addr, byte);
                let lcd_guard = match self.lcd.try_lock() {
                    Ok(lcd_guard) => lcd_guard,
                    Err(_) => return,
                };
                let mut lcd = match lcd_guard.try_borrow_mut() {
                    Ok(lcd) => lcd,
                    Err(_) => return,
                };
                match addr {
                    LCDC => {
                        lcd.lcd_control = byte;
                        if !lcd.lcd_control.get_bit(7) {
                            lcd.lcd_y = 0;
                        }
                    }
                    SCROLL_Y => lcd.scroll_y = byte,
                    SCROLL_X => lcd.scroll_x = byte,
                    LCD_Y => lcd.lcd_y = byte,
                    0xFF45 => lcd.lcd_y_cmp = byte,
                    PALLETE => lcd.background_pallete = byte,
                    0xFF4A => lcd.window_y = byte,
                    0xFF4B => lcd.window_x = byte,
                    _ => {}
                }
            }
            // Interrupt Flag (IF)
            0xFF0F => {
                trace!("IF register write @{:#X}: {:#X}", addr, byte);
                let interrupts_guard = match self.interrupts.try_lock() {
                    Ok(interrupts_guard) => interrupts_guard,
                    Err(_) => return,
                };
                let mut interrupts = match interrupts_guard.try_borrow_mut() {
                    Ok(interrupts) => interrupts,
                    Err(_) => return,
                };

                interrupts.set_interrupt_flag(byte);
            }
            // Interrupt Enable (IE)
            0xFFFF => {
                trace!("IE register write @{:#X}: {:#X}", addr, byte);
                let interrupts_guard = match self.interrupts.try_lock() {
                    Ok(interrupts_guard) => interrupts_guard,
                    Err(_) => return,
                };
                let mut interrupts = match interrupts_guard.try_borrow_mut() {
                    Ok(interrupts) => interrupts,
                    Err(_) => return,
                };

                interrupts.set_interrupt_enable(byte);
            }
            // I/O registers
            0xFF00..=0xFF7F => {
                warn!("Unimplemented IO register write @{:#X}: {:#X}", addr, byte);
            }
            // High Ram
            0xFF80..=0xFFFE => {
                trace!("HRAM write @{:#X}: {:#X}", addr, byte);
                let hram_guard = match self.hram.try_lock() {
                    Ok(hram_guard) => hram_guard,
                    Err(_) => return,
                };
                let mut hram = match hram_guard.try_borrow_mut() {
                    Ok(hram) => hram,
                    Err(_) => return,
                };
                hram[addr as usize - 0xFF80] = byte
            }
            _ => panic!("Illegal memory write at {:#X}", addr),
        }
    }

    pub fn request_interrupt(&self, interrupt: Interrupt) {
        let interrupts_guard = match self.interrupts.try_lock() {
            Ok(interrupts_guard) => interrupts_guard,
            Err(_) => return,
        };
        let mut interrupts = match interrupts_guard.try_borrow_mut() {
            Ok(interrupts) => interrupts,
            Err(_) => return,
        };

        match interrupt {
            Interrupt::VBlank => interrupts.vblank_requested = true,
            Interrupt::LCDStat => interrupts.lcd_stat_requested = true,
            Interrupt::Timer => interrupts.timer_requested = true,
            Interrupt::Serial => interrupts.serial_requested = true,
            Interrupt::Joypad => interrupts.joypad_requested = true,
        }
    }
}
