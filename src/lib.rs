#![no_std]
#![feature(core_intrinsics)]
#![allow(clippy::disallowed_methods, clippy::single_match)]
#![allow(dead_code, unused_variables, unused_imports, internal_features)]
extern crate alloc;

use core::fmt::Debug;
use crate::gametank_bus::Bus;
#[allow(clippy::unusual_byte_groupings)]

pub mod color_map;
pub mod blitter;
pub mod gametank_bus;
pub mod cartridges;
pub mod emulator;
pub mod inputs;
