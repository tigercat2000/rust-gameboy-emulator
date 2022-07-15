use bit_field::BitField;
use egui::Color32;
use tracing::{debug, info, trace};

use crate::{memory_bus::MemoryBus, GAMEBOY_HEIGHT, GAMEBOY_WIDTH};

pub type FrameBuffer = [u8; GAMEBOY_HEIGHT * GAMEBOY_WIDTH];

#[derive(Debug)]
pub struct PPU;

impl PPU {
    pub fn tick(&mut self, memory_bus: &MemoryBus) {
        let lcd_control = memory_bus.get_u8(0xFF40);
        if !lcd_control.get_bit(7) {
            trace!("LCD control disabled, skipping tick: {:#X}", lcd_control);
            return;
        }

        let lcd_y = memory_bus.get_u8(0xFF44);
        trace!("Running at y: {:#X}", lcd_y);
        trace!("Writeback: {:#X}", lcd_y.wrapping_add(1) % 0xA0);
        memory_bus.write_u8(0xFF44, lcd_y.wrapping_add(1) % 0xA0);
    }

    pub fn render(&mut self, memory_bus: &MemoryBus, frame_buffer: &mut FrameBuffer) {
        (0x9800u16..=0x9BFFu16)
            .into_iter()
            .map(|addr| memory_bus.get_u8(addr))
            .map(|index| {
                let base_address = if memory_bus.get_u8(0xFF40).get_bit(4) {
                    0x8000
                } else {
                    0x9000
                };

                let address_offset = if base_address == 0x8000 {
                    index as u16 * 16
                } else {
                    (index as i8 as i16 + 128) as u16
                };

                base_address + address_offset
            })
            .map(|tile_address| {
                let mut pixels: Vec<u8> = vec![];
                for line in 0..8 {
                    let lsb_byte = memory_bus.get_u8(tile_address + line * 2);
                    let msb_byte = memory_bus.get_u8(tile_address + line * 2 + 1);
                    for pixel in 0..8 {
                        let color_id =
                            (msb_byte.get_bit(pixel) as u8) << 1u8 | lsb_byte.get_bit(pixel) as u8;

                        let pallete = memory_bus.get_u8(0xFF47);
                        let remap = match color_id {
                            0 => pallete.get_bits(0..2),
                            1 => pallete.get_bits(2..4),
                            2 => pallete.get_bits(4..6),
                            3 => pallete.get_bits(6..8),
                            _ => unreachable!(),
                        };

                        pixels.push(match remap {
                            0 => 255,
                            1 => 192,
                            2 => 95,
                            3 => 0,
                            _ => unreachable!(),
                        });
                    }
                }

                pixels
            })
            .enumerate()
            .for_each(|(index, tile)| {
                let origin_x = (index * 8) % 256;
                let origin_y = ((index * 8) / 256) * 8;

                info!("Tile {} goes at ({}, {})", index, origin_x, origin_y);

                if origin_x >= 160 || origin_y >= 140 {
                    // reject for now
                    return;
                }

                for y in 0..8 {
                    for x in 0..8 {
                        if index == 544 {
                            info!("Laying out ({x}, {y})");
                        }

                        let color = tile[x + (y * 8)];
                        frame_buffer[(origin_x + x) + ((origin_y + y) * 160)] = color;
                    }
                }
            });
    }
}
