use bit_field::BitField;

use tracing::{debug, trace};

use crate::emulator::{
    memory_bus::{MemoryBus, LCDC, LCD_Y, PALLETE, SCROLL_X, SCROLL_Y},
    GAMEBOY_HEIGHT, GAMEBOY_WIDTH,
};

use super::memory_bus::Interrupt;

pub type FrameBuffer = [u8; GAMEBOY_HEIGHT * GAMEBOY_WIDTH];

#[derive(Debug, Default)]
pub struct PPU {
    pub updated: bool,
    mode_clock: u32,
    mode: u8,
    hblanking: bool,
}

impl PPU {
    /// Ticks in T-cycles
    pub fn tick(&mut self, memory_bus: &MemoryBus, frame_buffer: &mut FrameBuffer, ticks: u32) {
        let lcd_control = memory_bus.get_u8(LCDC);
        if !lcd_control.get_bit(7) {
            trace!("LCD control disabled, skipping tick: {:#X}", lcd_control);
            return;
        }

        self.hblanking = false;

        let mut ticks_left = ticks;
        let mut lcd_y = memory_bus.get_u8(LCD_Y);
        debug!("Running at {:#X} for {:#X} ticks", lcd_y, ticks_left);
        while ticks_left > 0 {
            let cur_ticks = ticks_left.min(80);
            self.mode_clock += cur_ticks;
            ticks_left -= cur_ticks;

            if self.mode_clock >= 456 {
                self.mode_clock -= 456;
                lcd_y = (lcd_y + 1) % 154;
                memory_bus.write_u8(LCD_Y, lcd_y);

                if lcd_y >= 144 {
                    self.change_mode_if_necessary(1, memory_bus, frame_buffer);
                }
            }

            if lcd_y < 144 {
                match self.mode_clock {
                    0..=80 => self.change_mode_if_necessary(2, memory_bus, frame_buffer),
                    81..=252 => self.change_mode_if_necessary(3, memory_bus, frame_buffer),
                    _ => self.change_mode_if_necessary(0, memory_bus, frame_buffer),
                }
            }
        }
    }

    fn change_mode_if_necessary(
        &mut self,
        mode: u8,
        memory_bus: &MemoryBus,
        frame_buffer: &mut FrameBuffer,
    ) {
        if self.mode != mode {
            self.change_mode(mode, memory_bus, frame_buffer);
        }
    }

    fn change_mode(&mut self, mode: u8, memory_bus: &MemoryBus, frame_buffer: &mut FrameBuffer) {
        self.mode = mode;

        match self.mode {
            0 => {
                self.render_scanline(memory_bus, frame_buffer);
                self.hblanking = true;
            }
            1 => {
                memory_bus.request_interrupt(Interrupt::VBlank);
                self.updated = true;
            }
            _ => {}
        }
    }

    fn render_scanline(&mut self, memory_bus: &MemoryBus, frame_buffer: &mut FrameBuffer) {
        for x in 0..GAMEBOY_WIDTH {
            self.set_color(x, 255, memory_bus, frame_buffer);
        }
        self.draw_bg(memory_bus, frame_buffer);
    }

    fn set_color(
        &mut self,
        x: usize,
        color: u8,
        memory_bus: &MemoryBus,
        frame_buffer: &mut FrameBuffer,
    ) {
        frame_buffer[memory_bus.get_u8(LCD_Y) as usize * GAMEBOY_WIDTH as usize + x] = color;
    }

    fn draw_bg(&mut self, memory_bus: &MemoryBus, frame_buffer: &mut FrameBuffer) {
        let lcd_control = memory_bus.get_u8(LCDC);
        if !lcd_control.get_bit(0) {
            trace!("Skipping Background due to LCDC0");
            return;
        }

        let lcd_y = memory_bus.get_u8(LCD_Y);

        let bg_y = memory_bus.get_u8(SCROLL_Y).wrapping_add(lcd_y);
        let bg_tile_y = (bg_y as u16 >> 3) & 31;

        for x in 0..GAMEBOY_WIDTH {
            let bg_x = memory_bus.get_u8(SCROLL_X) as u32 + x as u32;
            trace!("X: {:#X}, BGX: {:#X}", x, bg_x);

            let (tile_map_base, tile_y, tile_x, pixel_y, pixel_x) = {
                let base_addr = if lcd_control.get_bit(3) {
                    0x9C00
                } else {
                    0x9800
                };

                (
                    base_addr,
                    bg_tile_y,
                    (bg_x as u16 >> 3) & 31,
                    bg_y as u16 & 0x07,
                    bg_x as u16 & 0x07,
                )
            };

            trace!(
                "TMB: {:#X}, TY: {:#X}, TX: {:#X}, IDX: {:#X}",
                tile_map_base,
                tile_y,
                tile_x,
                tile_map_base + tile_y * 32 + tile_x
            );

            let tile_number = memory_bus.get_u8(tile_map_base + tile_y * 32 + tile_x);

            let tile_address = {
                let base_address = if lcd_control.get_bit(4) {
                    0x8000
                } else {
                    0x8800
                };

                let address_offset = if base_address == 0x8000 {
                    tile_number as u16 * 16
                } else {
                    (tile_number as i8 as i16 + 128) as u16 * 16
                };

                (base_address + address_offset) as u16
            };

            let tile_pixel = tile_address + (pixel_y * 2);
            let lsb_byte = memory_bus.get_u8(tile_pixel);
            let msb_byte = memory_bus.get_u8(tile_pixel + 1);

            let color_id = (msb_byte.get_bit(7 - pixel_x as usize) as u8) << 1
                | lsb_byte.get_bit(7 - pixel_x as usize) as u8;

            let pallete = memory_bus.get_u8(PALLETE);
            let remap = match color_id {
                0 => pallete.get_bits(0..2),
                1 => pallete.get_bits(2..4),
                2 => pallete.get_bits(4..6),
                3 => pallete.get_bits(6..8),
                _ => unreachable!(),
            };

            let color = match remap {
                0 => 255,
                1 => 192,
                2 => 95,
                3 => 0,
                _ => unreachable!(),
            };

            self.set_color(x, color, memory_bus, frame_buffer);
        }
    }
}
