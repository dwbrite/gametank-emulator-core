use std::intrinsics::transmute;
use std::ops::{Deref, DerefMut};
use crate::cartridges::Cartridge;


#[derive(Debug, Clone)]
pub struct Cartridge2M {
    data: Box<[[u8; 0x4000]; 128]>,
    pub bank_shifter: u8,
    pub bank_mask: u16,
}

impl Cartridge for Cartridge2M {
    fn from_slice(slice: &[u8]) -> Self {
        let mut data = [0u8; 0x4000*128];
        data.copy_from_slice(&slice);
        let data: Box<[[u8; 0x4000]; 128]> = unsafe { Box::new(transmute(data)) };
        Self {
            data,
            bank_shifter: 0,
            bank_mask: 0x7E,
        }
    }

    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x4000..=0x7FFF => {
                self.data[0x7F][address as usize & 0x3FFF]
            }
            0x0000..=0x3FFF => {
                self.data[self.bank_mask as usize & 0x7F][address as usize & 0x3FFF]
            }
            _ => { panic!("how the hell did you get here?"); }
        }
        // self.data[]
    }
}
