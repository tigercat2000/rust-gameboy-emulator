use std::io::Read;

use bit_field::BitField;
use tracing::{error, trace, warn};

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

#[derive(Clone, Copy, Debug)]
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

impl Interrupt {
    pub fn addr(&self) -> u16 {
        match self {
            Interrupt::VBlank => 0x0040,
            Interrupt::LCDStat => 0x0048,
            Interrupt::Timer => 0x0050,
            Interrupt::Serial => 0x0058,
            Interrupt::Joypad => 0x0060,
        }
    }
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
    /// HACK: TODO: Remove when doing MBC
    // fake_cartram: [u8; 0xBFFF - 0xA000 + 1],
    wram1: [u8; 0xCFFF - 0xC000 + 1],
    wram2: [u8; 0xDFFF - 0xD000 + 1],
    vram: [u8; 0x1FFF + 1],
    oam: [u8; 0xFE9F - 0xFE00 + 1],
    hram: [u8; 0xFFFE - 0xFF80 + 1],
    lcd: LCD,
    interrupts: Interrupts,
    console_buffer: String,
}

impl MemoryBus {
    pub fn new<R: Read>(mut reader: R) -> Self {
        let mut vec = Vec::new();
        reader.read_to_end(&mut vec).unwrap();
        Self {
            program: vec,
            // fake_cartram: [0; 0xBFFF - 0xA000 + 1],
            wram1: [0; 0xCFFF - 0xC000 + 1],
            wram2: [0; 0xDFFF - 0xD000 + 1],
            vram: [0; 0x1FFF + 1],
            oam: [0; 0xFE9F - 0xFE00 + 1],
            hram: [0; 0xFFFE - 0xFF80 + 1],
            lcd: LCD::default(),
            interrupts: Interrupts::default(),
            console_buffer: String::new(),
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => {
                trace!("PROG read @{:#X}", addr);
                self.program[addr as usize]
            }
            0x8000..=0x9FFF => self.vram[addr as usize - 0x8000],
            // TODO: Remove when MBC
            // 0xA000..=0xBFFF => self.fake_cartram[addr as usize - 0xA000],
            // WRAM 1
            0xC000..=0xCFFF => {
                let val = self.wram1[addr as usize - 0xC000];
                trace!("WRAM read @{:#X}: {:#X}", addr, val);
                val
            }
            // WRAM 2
            0xD000..=0xDFFF => {
                let val = self.wram2[addr as usize - 0xD000];
                trace!("WRAM read @{:#X}: {:#X}", addr, val);
                val
            }
            // ECHO RAM
            0xE000..=0xFDFF => {
                warn!(
                    "(continuing) Illegal read from ECHO RAM @{:#X} Reroute: {:#X}",
                    addr,
                    addr - 0x2000
                );
                self.read_u8(addr - 0x2000)
            }
            // OAM
            0xFE00..=0xFE9F => {
                let val = self.oam[addr as usize - 0xFE00];
                trace!("OAM read @{:#X}: {:#X}", addr, val);
                val
            }
            // Joypad
            0xFF00 => 0b1111_1111,
            0xFF40..=0xFF4B => {
                trace!("LCD register read @{:#X}", addr);
                match addr {
                    LCDC => self.lcd.lcd_control,
                    SCROLL_Y => self.lcd.scroll_y,
                    SCROLL_X => self.lcd.scroll_x,
                    LCD_Y => self.lcd.lcd_y,
                    0xFF45 => self.lcd.lcd_y_cmp,
                    PALLETE => self.lcd.background_pallete,
                    0xFF4A => self.lcd.window_y,
                    0xFF4B => self.lcd.window_x,
                    _ => {
                        error!("Attempted to read unimplemented LCD register {:#X}", addr);
                        unimplemented!()
                    }
                }
            }
            // Interrupt Flag (IF)
            IF => {
                let val = self.interrupts.get_interrupt_flag();
                trace!("IF read @{:#X}: {:#X}", addr, val);
                val
            }
            // Interrupt Enable (IE)
            IE => {
                let val = self.interrupts.get_interrupt_enable();
                trace!("IE read @{:#X}: {:#X}", addr, val);
                val
            }
            0xFF01..=0xFF7F => {
                warn!("Unimplemented IO register read @{:#X}", addr);
                0x00
            }
            0xFF80..=0xFFFE => {
                let val = self.hram[addr as usize - 0xFF80];
                trace!("HRAM read @{:#X}: {:#X}", addr, val);
                val
            }
            other => {
                error!("Attempt to read unimplemented memory {:#X}", other);
                unimplemented!()
            }
        }
    }

    #[allow(clippy::identity_op)]
    pub fn get_instr(&self, addr: u16) -> [u8; 4] {
        [
            self.read_u8(addr + 0),
            self.read_u8(addr + 1),
            self.read_u8(addr + 2),
            self.read_u8(addr + 3),
        ]
    }

    pub fn write_u8(&mut self, addr: u16, byte: u8) {
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
                self.vram[addr as usize - 0x8000] = byte
            }
            // TODO: Remove when MBC
            // 0xA000..=0xBFFF => self.fake_cartram[addr as usize - 0xA000] = byte,
            // WRAM 1
            0xC000..=0xCFFF => {
                trace!("WRAM write @{:#X}: {:#X}", addr, byte);
                self.wram1[addr as usize - 0xC000] = byte
            }
            // WRAM 2
            0xD000..=0xDFFF => {
                trace!("WRAM write @{:#X}: {:#X}", addr, byte);
                self.wram2[addr as usize - 0xD000] = byte
            }
            // ECHO RAM
            0xE000..=0xFDFF => {
                warn!(
                    "(continuing) Illegal write to ECHO RAM @{:#X}: {:#X} Reroute: {:#X}",
                    addr,
                    byte,
                    addr - 0x2000
                );
                self.write_u8(addr - 0x2000, byte)
            }
            // OAM
            0xFE00..=0xFE9F => {
                trace!("OAM write @{:#X}: {:#X}", addr, byte);
                self.oam[addr as usize - 0xFE00] = byte
            }
            0xFEA0..=0xFEFF => {
                warn!(
                    "(continuing) Illegal write to prohibited zone @{:#X}: {:#X}",
                    addr, byte
                );
            }
            // Joypad
            0xFF00 => {
                // Do nothing
            }
            // Serial
            0xFF01 => {
                let byte = byte as char;
                if byte == '\n' {
                    println!("{}", self.console_buffer);
                    self.console_buffer.clear();
                } else {
                    self.console_buffer.push(byte);
                }
            }
            // Serial Flush
            0xFF02 => {}
            // LCD
            0xFF40..=0xFF4B => {
                trace!("LCD register write @{:#X}: {:#X}", addr, byte);
                match addr {
                    LCDC => {
                        self.lcd.lcd_control = byte;
                        if !self.lcd.lcd_control.get_bit(7) {
                            self.lcd.lcd_y = 0;
                        }
                    }
                    SCROLL_Y => self.lcd.scroll_y = byte,
                    SCROLL_X => self.lcd.scroll_x = byte,
                    LCD_Y => self.lcd.lcd_y = byte,
                    0xFF45 => self.lcd.lcd_y_cmp = byte,
                    PALLETE => self.lcd.background_pallete = byte,
                    0xFF4A => self.lcd.window_y = byte,
                    0xFF4B => self.lcd.window_x = byte,
                    _ => {}
                }
            }
            // Interrupt Flag (IF)
            0xFF0F => {
                trace!("IF register write @{:#X}: {:#X}", addr, byte);
                self.interrupts.set_interrupt_flag(byte);
            }
            // Interrupt Enable (IE)
            0xFFFF => {
                trace!("IE register write @{:#X}: {:#X}", addr, byte);
                self.interrupts.set_interrupt_enable(byte);
            }
            // I/O registers
            0xFF03..=0xFF7F => {
                warn!("Unimplemented IO register write @{:#X}: {:#X}", addr, byte);
            }
            // High Ram
            0xFF80..=0xFFFE => {
                trace!("HRAM write @{:#X}: {:#X}", addr, byte);
                self.hram[addr as usize - 0xFF80] = byte
            }
            #[allow(unreachable_patterns)]
            // During periods where we remove some memory
            _ => unimplemented!("Unimplemented memory write at {:#X}", addr),
        }
    }

    // Stack Ops
    pub fn read_stack_16(&self, sp: &mut u16) -> u16 {
        let lower = self.read_stack(sp) as u16;
        let upper = self.read_stack(sp) as u16;
        upper << 8 | lower
    }

    pub fn read_stack(&self, sp: &mut u16) -> u8 {
        let val = self.read_u8(*sp);
        *sp = sp.wrapping_add(1);
        val
    }

    pub fn write_stack_16(&mut self, sp: &mut u16, word: u16) {
        self.write_stack(sp, word.get_bits(8..16) as u8);
        self.write_stack(sp, word.get_bits(0..8) as u8);
    }

    pub fn write_stack(&mut self, sp: &mut u16, byte: u8) {
        *sp = sp.wrapping_sub(1);
        self.write_u8(*sp, byte);
    }

    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        match interrupt {
            Interrupt::VBlank => self.interrupts.vblank_requested = true,
            Interrupt::LCDStat => self.interrupts.lcd_stat_requested = true,
            Interrupt::Timer => self.interrupts.timer_requested = true,
            Interrupt::Serial => self.interrupts.serial_requested = true,
            Interrupt::Joypad => self.interrupts.joypad_requested = true,
        }
    }

    pub fn reset_interrupt(&mut self, interrupt: Interrupt) {
        match interrupt {
            Interrupt::VBlank => self.interrupts.vblank_requested = false,
            Interrupt::LCDStat => self.interrupts.lcd_stat_requested = false,
            Interrupt::Timer => self.interrupts.timer_requested = false,
            Interrupt::Serial => self.interrupts.serial_requested = false,
            Interrupt::Joypad => self.interrupts.joypad_requested = false,
        }
    }

    pub fn get_next_interrupt(&self) -> Option<Interrupt> {
        if self.interrupts.vblank_requested && self.interrupts.vblank_enabled {
            return Some(Interrupt::VBlank);
        }
        if self.interrupts.lcd_stat_requested && self.interrupts.lcd_stat_enabled {
            return Some(Interrupt::LCDStat);
        }
        if self.interrupts.timer_requested && self.interrupts.timer_enabled {
            return Some(Interrupt::Timer);
        }
        if self.interrupts.serial_requested && self.interrupts.serial_enabled {
            return Some(Interrupt::Serial);
        }
        if self.interrupts.joypad_requested && self.interrupts.joypad_enabled {
            return Some(Interrupt::Joypad);
        }
        None
    }

    pub fn hram_dump(&self) {
        error!("{:#X?}", self.hram);
    }
}
