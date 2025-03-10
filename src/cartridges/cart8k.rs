use std::ops::{Deref, DerefMut};
use crate::cartridges::Cartridge;

#[derive(Debug, Clone)]
pub struct Cartridge8K {
    data: Box<[u8; 0x8000]>
}

impl Deref for Cartridge8K {
    type Target = [u8; 0x8000];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Cartridge8K {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl Cartridge for Cartridge8K {
    fn from_slice(slice: &[u8]) -> Self {
        let mut data = [0; 0x8000];
        data[0x6000..0x8000].copy_from_slice(&slice);
        Self {
            data: Box::new(data),
        }
    }

    fn read_byte(&self, address: u16) -> u8 {
        self.data[address as usize]
    }
}
