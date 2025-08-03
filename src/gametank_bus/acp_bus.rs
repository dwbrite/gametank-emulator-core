use alloc::boxed::Box;
use log::{error};
use w65c02s::{System, W65C02S};
use crate::gametank_bus::Bus;

pub static mut aram2: [u8; 0x1000] = [0; 0x1000];


#[derive(Default, Debug)]
pub struct AcpBus {
    pub irq_counter: i32,
    pub sample: u8,
}

impl System for AcpBus {
    #[inline(always)]
    fn read(&mut self, _: &mut W65C02S, address: u16) -> u8 {
        unsafe { *aram2.get_unchecked((address as usize) & 0x0FFF) }
    }

    #[inline(always)]
    fn write(&mut self, _: &mut W65C02S, address: u16, data: u8) {
        if address >= 0x8000 {
            self.sample = data;
        }

        unsafe { *aram2.get_unchecked_mut((address as usize) & 0x0FFF) = data; }
    }
}