use bit_field::BitField;
use tracing::{debug, trace};

use crate::memory_bus::MemoryBus;

#[derive(Debug)]
pub struct PPU {
    frame_buffer: [u8; 160 * 140],
}

impl PPU {
    pub fn tick(&mut self, memory_bus: &MemoryBus) {
        let lcd_control = memory_bus.get_u8(0xFF40);
        if !lcd_control.get_bit(7) {
            trace!("LCD control disabled, skipping tick: {:#X}", lcd_control);
            return;
        }

        let lcd_y = memory_bus.get_u8(0xFF44);
        trace!("Running at y: {:#X}", lcd_y);
        debug!("Writeback: {:#X}", lcd_y.wrapping_add(1) % 0xA0);
        memory_bus.write_u8(0xFF44, lcd_y.wrapping_add(1) % 0xA0);
    }

    pub fn get_frame_buffer(&self) -> &[u8] {
        self.frame_buffer.as_slice()
    }
}

impl Default for PPU {
    fn default() -> Self {
        Self {
            frame_buffer: [0; 160 * 140],
        }
    }
}
