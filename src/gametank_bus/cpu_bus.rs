use alloc::boxed::Box;
use core::cell::Ref;
use w65c02s::{System, W65C02S};
use rand::{Rng};
use log::{debug, warn};
use crate::cartridges::CartridgeType;
use crate::gametank_bus::{aram2, Bus};
use crate::gametank_bus::reg_system_control::*;
use crate::inputs::GamePad;
use crate::gametank_bus::cpu_bus::ByteDecorator::{AudioRam, CpuStack, SystemRam, Unreadable, Vram, ZeroPage};
use crate::gametank_bus::reg_blitter::{BlitStart, BlitterRegisters};
use crate::gametank_bus::reg_etc::{new_framebuffer, BankingRegister, BlitterFlags, FrameBuffer, GraphicsMemoryMap, SharedFrameBuffer};
use crate::gametank_bus::reg_system_control::*;

const CURRENT_GAME: &[u8] = include_bytes!("../cubicle.gtr");

#[derive(Copy, Clone, Debug)]
pub enum ByteDecorator {
    ZeroPage(u8),
    CpuStack(u8),
    SystemRam(u8),
    // SCR(u8),
    // VersatileInterfaceAdapter(u8),
    AudioRam(u8),
    Vram(u8),
    Framebuffer(u8),
    // Blitter(u8),
    Aram(u8),
    Unreadable(u8),
}

// pub static mut ram_banks2: &'static mut [[u8; 0x2000]; 4] = &mut [[0; 0x2000]; 4];
// pub static mut framebuffers: &'static mut [[u8; 0x4000]; 2] = &mut [[0x69; 0x4000]; 2];

#[derive(Debug)]
pub struct CpuBus {
    pub system_control: SystemControl,
    pub blitter: BlitterRegisters,

    // heap allocations to prevent stackoverflow, esp on web
    pub ram_banks: Box<[[u8; 0x2000]; 4]>,
    pub framebuffers: [SharedFrameBuffer; 2],
    pub vram_banks: Box<[[u8; 256*256]; 8]>,

    pub vram_quad_written: [bool; 32],

    // pub aram: Option<ARAM>,
    pub cartridge: CartridgeType,
}

impl Default for CpuBus {
    fn default() -> Self {
        // TODO: re-add rng to initial framebuffer?

        let bus = Self {
            system_control: SystemControl {
                reset_acp: 0,
                nmi_acp: 0,
                banking_register: BankingRegister(0),
                via_regs: [0; 16],
                audio_enable_sample_rate: 0,
                dma_flags: BlitterFlags(0b0111_1111),
                gamepads: [GamePad::default(), GamePad::default()]
            },
            blitter: BlitterRegisters {
                vx: 0,
                vy: 0,
                gx: 0,
                gy: 0,
                width: 127,
                height: 127,
                start: BlitStart {
                    write: 0,
                    addressed: false,
                },
                color: 0b101_00_000, // offwhite
            },
            ram_banks: Box::new([[0; 0x2000]; 4]), // 16k ram
            framebuffers: [new_framebuffer(0x00), new_framebuffer(0xFF)], // 128*128*2 = 32k framebuffer
            vram_banks: Box::new([[0; 256*256]; 8]), // 64k * 8 = 512k
            cartridge: CartridgeType::from_slice(CURRENT_GAME),
            // aram: Some(Box::new([0; 0x1000])), // audio ram is 2k
            vram_quad_written: [false; 32],
        };

/*        for p in bus.framebuffers[0].borrow_mut().iter_mut() {
            // *p = rng.gen();
        }*/

        // for p in bus.framebuffers[1].borrow_mut().iter_mut() {
        //     // *p = rng.gen();
        // }

        bus
    }
}

impl CpuBus {
    pub fn read_full_framebuffer(&self) -> Ref<'_, FrameBuffer> {
        let fb = self.system_control.get_framebuffer_out();
        self.framebuffers[fb].borrow()
        // unsafe { framebuffers.get_unchecked_mut(fb) }
    }

    fn update_flash_shift_register(&mut self, next_val: u8) {
        // match &mut self.cartridge {
        //     CartridgeType::Cart2m(cartridge) => {
        //         // For now, assuming that if we're using Flash2M hardware, we're behaving ourselves
        //         let old_val = self.system_control.via_regs[VIA_IORA]; // Get the previous value from the VIA
        //         let rising_bits = next_val & !old_val;
        //
        //         if rising_bits & VIA_SPI_BIT_CLK != 0 {
        //             cartridge.bank_shifter <<= 1; // Shift left
        //             cartridge.bank_shifter &= 0xFE; // Ensure the last bit is cleared
        //             cartridge.bank_shifter |= ((old_val & VIA_SPI_BIT_MOSI) != 0) as u8; // Set the last bit based on MOSI
        //         } else if rising_bits & VIA_SPI_BIT_CS != 0 {
        //             // Flash cart CS is connected to latch clock
        //             if (cartridge.bank_mask ^ cartridge.bank_shifter as u16) & 0x80 != 0 {
        //                 // TODO: support saving
        //                 // self.save_nvram(); // Assuming this is defined elsewhere or is a method within CpuBus
        //                 warn!("Saving is not yet supported");
        //             }
        //             cartridge.bank_mask = cartridge.bank_shifter as u16; // Update the bank mask
        //             debug!("Flash bank mask set to 0x{:x}", cartridge.bank_mask);
        //         }
        //     },
        //     _ => {} // do nothing
        // }
    }


    #[inline]
    pub fn write_byte(&mut self, address: u16, data: u8) {
        match address {
            // system RAM
            0x0000..=0x1FFF => {
                unsafe {
                    *self.ram_banks.get_unchecked_mut(self.system_control.get_ram_bank()).get_unchecked_mut(address as usize) = data;
                }
                // println!("${:04X}={:02X}", address, data);
            }

            // system control registers
            0x2000..=0x2009 => {
                self.system_control.write_byte(address, data);
                // println!("${:04X}={:08b}", address, data);
            }

            // versatile interface adapter (GPIO, timers)
            0x2800..=0x280F => {
                let register = (address & 0xF) as usize;
                match (address & 0xF) as usize {
                    VIA_IORA => {
                        self.update_flash_shift_register(data);
                    }
                    _ => {}
                }
                self.system_control.via_regs[register] = data;
            }

            // audio RAM
            0x3000..=0x3FFF => unsafe {
                *aram2.get_unchecked_mut((address - 0x3000) as usize) = data;
            }

            // VRAM/Framebuffer/Blitter
            0x4000..=0x7FFF => {
                match self.system_control.get_graphics_memory_map() {
                    GraphicsMemoryMap::FrameBuffer => unsafe {
                        let fb = self.system_control.banking_register.framebuffer() as usize;
                        *self.framebuffers.get_unchecked_mut(fb).borrow_mut().get_unchecked_mut(address as usize - 0x4000) = data;
                    }
                    GraphicsMemoryMap::VRAM => {
                        let vram_page = self.system_control.banking_register.vram_page() as usize;
                        let quadrant = self.blitter.vram_quadrant();
                        self.vram_banks[vram_page][address as usize - 0x4000 + quadrant*(128*128)] = data;
                        self.vram_quad_written[quadrant + vram_page * 4] = true;
                    }
                    GraphicsMemoryMap::BlitterRegisters => {
                        self.blitter.write_byte(address, data);
                        // println!("blitter reg write -> ${:04X}={:02X}", address, data);
                    }
                }
            }
            _ => {
                warn!("Attempted to write read-only memory at: ${:02X}", address);
            }
        }
    }

    #[inline]
    pub fn read_byte(&mut self, address: u16) -> u8 {
        if address >= 0x8000 {
            return self.cartridge.read_byte(address - 0x8000);
        }
        if address < 0x2000 {
            return unsafe { *self.ram_banks.get_unchecked(self.system_control.get_ram_bank()).get_unchecked(address as usize) };
        }

        unsafe { self.read_byte_slow(address) }
    }

    unsafe fn read_byte_slow(&mut self, address: u16) -> u8 {
        match address {
            // system control registers
            0x2000..=0x2009 => {
                return self.system_control.read_byte(address);
            }

            // versatile interface adapter (GPIO, timers)
            0x2800..=0x280F => {
                let register = (address & 0xF) as usize;
                return *self.system_control.via_regs.get_unchecked(register) ;
            }

            // audio RAM
            0x3000..=0x3FFF => {
                return *aram2.get_unchecked((address - 0x3000) as usize);
            }

            // VRAM/Framebuffer/Blitter
            0x4000..=0x7FFF => {
                match self.system_control.get_graphics_memory_map() {
                    GraphicsMemoryMap::FrameBuffer => {
                        let fb = self.system_control.banking_register.framebuffer() as usize;
                        return *self.framebuffers.get_unchecked(fb).borrow().get_unchecked(address as usize - 0x4000);
                    }
                    GraphicsMemoryMap::VRAM => {
                        let vram_page = self.system_control.banking_register.vram_page() as usize;
                        let quadrant = self.blitter.vram_quadrant();
                        return *self.vram_banks.get_unchecked(vram_page).get_unchecked(address as usize - 0x4000 + quadrant * (128 * 128));
                    }
                    GraphicsMemoryMap::BlitterRegisters => {
                        return self.blitter.read_byte(address);
                    }
                }
            }
            _ => {
                warn!("Attempted to inaccessible memory at: ${:02X}", address);
            }
        }

        0
    }

    #[inline(always)]
    // pub fn peek_byte_decorated(&self, address: u16) -> ByteDecorator {
    //     match address {
    //         0x0000..=0x00FF => { ZeroPage(self.ram_banks[self.system_control.get_ram_bank()][address as usize]) },
    //         0x0100..=0x01FF => { CpuStack(self.ram_banks[self.system_control.get_ram_bank()][address as usize]) },
    //         0x0200..=0x1FFF => { SystemRam(self.ram_banks[self.system_control.get_ram_bank()][address as usize]) },
    //         0x2000..=0x2009 => { Unreadable(self.system_control.peek_byte(address)) },
    //         // 0x2800..=0x280F => { Via(self.system_control.via_regs[(address & 0xF) as usize]) },
    //         0x3000..=0x3FFF => { AudioRam(if let Some(aram) = &self.aram { aram[(address - 0x3000) as usize] } else { 0 }) },
    //         0x4000..=0x7FFF => {
    //             match self.system_control.get_graphics_memory_map() {
    //                 GraphicsMemoryMap::FrameBuffer => {
    //                     let fb = self.system_control.banking_register.framebuffer() as usize;
    //                     ByteDecorator::Framebuffer(self.framebuffers[fb].borrow()[address as usize - 0x4000])
    //                 }
    //                 GraphicsMemoryMap::VRAM => {
    //                     let vram_page = self.system_control.banking_register.vram_page() as usize;
    //                     let quadrant = self.blitter.vram_quadrant();
    //                     Vram(self.vram_banks[vram_page][address as usize - 0x4000 + quadrant*(128*128)])
    //                 }
    //                 GraphicsMemoryMap::BlitterRegisters => {
    //                     Unreadable(0)
    //                 }
    //             }
    //         },
    //         _ => Unreadable(0),
    //     }
    // }

    pub fn vblank_nmi_enabled(&self) -> bool {
        self.system_control.dma_flags.dma_nmi()
    }
}

impl System for CpuBus {

    #[inline(always)]
    fn read(&mut self, _: &mut W65C02S, addr: u16) -> u8 {
        self.read_byte(addr)
    }


    #[inline(always)]
    fn write(&mut self, _: &mut W65C02S, addr: u16, data: u8) {
        self.write_byte(addr, data);
    }
}
