#![allow(dead_code, unused_variables, unused_imports, internal_features, static_mut_refs)]

pub mod cart8k;
pub mod cart32k;
pub mod cart2m;

use alloc::boxed::Box;
use log::error;
use crate::cartridges::cart2m::Cartridge2M;
use crate::cartridges::cart8k::Cartridge8K;
use crate::cartridges::cart32k::{Cartridge32K};

pub trait Cartridge {
    fn from_slice(slice: &[u8]) -> Self;
    fn read_byte(&self, address: u16) -> u8;
    fn write_byte(&mut self, address: u16, data: u8) {
        //default impl do nothing
    }
}

#[derive(Debug, Clone)]
pub enum CartridgeType {
    Cart8k(Cartridge8K),
    Cart32k(Cartridge32K),
    Cart2m(Box<Cartridge2M>),
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
            0x200000 => {
                CartridgeType::Cart2m(Box::new(Cartridge2M::from_slice(slice)))
            }
            _ => {
                panic!("unimplemented");
            }
        }
    }

    #[inline(always)]
    pub fn read_byte(&self, address: u16) -> u8 {
        match self {
            CartridgeType::Cart8k(c) => {c.read_byte(address)}
            CartridgeType::Cart32k(c) => {c.read_byte(address)}
            CartridgeType::Cart2m(c) => {c.read_byte(address)}
        }
    }

    pub fn write_byte(&mut self, address: u16, data: u8) {
        match self {
            CartridgeType::Cart2m(c) => { c.write_byte(address, data) }
            _ => { error!("attempted write to non-writable cartridge") }
        }
    }
}
