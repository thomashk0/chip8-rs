#![no_std]
// #![feature(maybe_uninit_ref)]

use core::mem::MaybeUninit;
use core::panic::PanicInfo;

use chip8::{CHIP8_FB_W, Chip8Emulator, Chip8Fb};
use chip8::screen::CHIP8_FB_H;

const EMU_CPU_HZ: u32 = 600;

static mut FB: Chip8Fb = [0; CHIP8_FB_W * CHIP8_FB_H];

static mut EMU: MaybeUninit::<Chip8Emulator> = MaybeUninit::uninit();

static PARTICLES_ROM: &'static [u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/roms/invaders.ch8"));

#[no_mangle]
pub unsafe extern fn chip8_init() {
    chip8_reset();
}

#[no_mangle]
pub unsafe extern fn chip8_reset() {
    let emu = &mut *EMU.as_mut_ptr();
    *emu = Chip8Emulator::new(EMU_CPU_HZ);
    emu.peripherals_mut().screen.set_inverted_y(false);
    emu.load_rom(PARTICLES_ROM);
}

#[no_mangle]
pub unsafe extern fn chip8_advance_ms(ms: u32) -> bool {
    let emu = &mut *EMU.as_mut_ptr();
    FB.copy_from_slice(emu.framebuffer());
    for x in FB.iter_mut() {
        *x |= 0xFF_00_00_00;
    }
    match emu.advance_ms(ms) {
        Ok(()) => true,
        Err(_) => false
    }
}

#[no_mangle]
pub unsafe extern fn chip8_key_down(k: u32) {
    let emu = &mut *EMU.as_mut_ptr();
    emu.peripherals_mut().keypad.key_pressed(k as u8);
}

#[no_mangle]
pub unsafe extern fn chip8_key_up(k: u32) {
    let emu = &mut *EMU.as_mut_ptr();
    emu.peripherals_mut().keypad.key_released(k as u8);
}

#[no_mangle]
pub unsafe extern fn chip8_fb() -> &'static [u32; CHIP8_FB_W * CHIP8_FB_H] {
    &FB
}


#[no_mangle]
pub unsafe extern fn chip8_fb_width() -> u32 {
    CHIP8_FB_W as u32
}

#[no_mangle]
pub unsafe extern fn chip8_fb_height() -> u32 {
    CHIP8_FB_H as u32
}

#[panic_handler]
fn handle_panic(_: &PanicInfo) -> ! {
    loop {}
}
