#![cfg_attr(not(feature = "std"), no_std)]

pub mod cpu;
pub mod screen;
pub mod keypad;
pub mod emu;
pub mod utils;

pub use cpu::{Insn, Chip8Cpu};
pub use emu::{Chip8Emulator, Chip8Peripherals, CHIP8_PERIPH_HZ};
pub use screen::{Screen, Chip8Fb, CHIP8_FB_W, CHIP8_FB_H};
pub use utils::Pcg32;
