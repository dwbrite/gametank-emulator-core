use alloc::boxed::Box;
use log::{error};
use w65c02s::{System, W65C02S};
use crate::gametank_bus::Bus;

pub static mut aram2: &'static mut [u8; 0x1000]  = &mut [0; 0x1000];

#[derive(Default, Debug)]
pub struct AcpBus {
    cycles: u8,
    pub irq_counter: i32,

    pub sample: u8,
}

impl AcpBus {
    #[inline(always)]
    pub(crate) fn write_byte(&mut self, address: u16, data: u8) {
        match address {
            0x0000..0x1000 => {
                unsafe { *aram2.get_unchecked_mut(address as usize) = data; }
            }
            0x8000..=0xFFFF => {
                self.sample = data;
            }
            _ => {}
        }
    }

    #[inline(always)]
    pub(crate) fn read_byte(&self, address: u16) -> u8 {
        unsafe { *aram2.get_unchecked((address as usize) % 0x1000) }
    }
}

impl System for AcpBus {
    #[inline(always)]
    fn read(&mut self, _: &mut W65C02S, addr: u16) -> u8 {
        self.read_byte(addr)
    }

    #[inline(always)]
    fn write(&mut self, _: &mut W65C02S, addr: u16, data: u8) {
        self.write_byte(addr, data);
    }
}