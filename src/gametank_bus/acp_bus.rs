use alloc::boxed::Box;
use log::{error};
use w65c02s::{System, W65C02S};
use crate::gametank_bus::Bus;

pub(crate) type ARAM = Box<[u8; 0x1000]>;

#[derive(Default, Debug)]
pub struct AcpBus {
    cycles: u8,
    pub irq_counter: i32,

    pub sample: u8,
    pub aram: Option<ARAM>
}

impl AcpBus {
    pub(crate) fn write_byte(&mut self, address: u16, data: u8) {
        if let Some(aram) = &mut self.aram {
            match address {
                0x0000..0x1000 => {
                    aram[address as usize] = data;
                }
                0x8000..=0xFFFF => {
                    self.sample = data;
                }
                _ => {}
            }
        } else {
            error!("acp contention on audio ram")
        }
    }

    pub(crate) fn read_byte(&self, address: u16) -> u8 {
        if let Some(aram) = &self.aram {
            aram[(address as usize) % 0x1000]
        } else {
            error!("acp contention on audio ram");
            0
        }
    }
}

impl System for AcpBus {
    fn read(&mut self, _: &mut W65C02S, addr: u16) -> u8 {
        self.cycles += 1;
        self.irq_counter -= 1;
        self.read_byte(addr)
    }

    fn write(&mut self, _: &mut W65C02S, addr: u16, data: u8) {
        self.cycles += 1;
        self.irq_counter -= 1;
        self.write_byte(addr, data);
    }
}

impl Bus for AcpBus {
    fn clear_cycles(&mut self) -> u8 {
        let ret = self.cycles;
        self.cycles = 0;
        ret
    }
}