pub mod cart8k;
pub mod cart32k;
pub mod cart2m;

use alloc::boxed::Box;
use crate::cartridges::cart8k::Cartridge8K;
use crate::cartridges::cart32k::{Cartridge32K};

pub trait Cartridge {
    fn from_slice(slice: &[u8]) -> Self;
    fn read_byte(&self, address: u16) -> u8;
}

#[derive(Debug, Clone)]
pub enum CartridgeType {
    Cart8k(Cartridge8K),
    Cart32k(Cartridge32K),
    // Cart2m(Box<Cartridge2M>),
}

impl CartridgeType {
    pub fn from_slice(slice: &[u8]) -> Self {
        match slice.len() {
            0x2000 => {
                CartridgeType::Cart8k(Cartridge8K::from_slice(slice))
            }
            0x8000 => {
                CartridgeType::Cart32k(Cartridge32K::from_slice(slice))
            }
            // 0x200000 => {
            //     CartridgeType::Cart2m(Box::new(Cartridge2M::from_slice(slice)))
            // }
            _ => {
                panic!("unimplemented");
            }
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match self {
            CartridgeType::Cart8k(c) => {c.read_byte(address)}
            CartridgeType::Cart32k(c) => {c.read_byte(address)}
            // CartridgeType::Cart2m(c) => {c.read_byte(address)}
        }
    }
}

//
// fn from_slice(slice: &[u8]) -> Box<dyn Cartridge> {
//
// }
