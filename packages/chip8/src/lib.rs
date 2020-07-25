#![cfg_attr(not(feature = "std"), no_std)]

pub mod cpu;
pub mod screen;
pub mod keypad;
pub mod emu;
pub mod utils;

pub use cpu::{Insn, Chip8Cpu};
pub use emu::{Chip8Emulator, Chip8Peripherals};
pub use screen::{Screen, Chip8Fb, CHIP8_FB_W};
pub use utils::Pcg32;
