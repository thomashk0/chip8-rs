#![no_std]
// #![feature(maybe_uninit_ref)]

use core::mem::MaybeUninit;
use core::panic::PanicInfo;

use chip8::{CHIP8_FB_W, CHIP8_FB_H, Chip8Emulator, Chip8Fb, CHIP8_PERIPH_HZ};

static mut EMU_CPU_HZ: u32 = 600;

// NOTE: the lower part of the memory is supposed to be reserved to the emulator
const CHIP8_MEM_SIZE: usize = 4096 - 0x200;

static mut FRAMEBUFFER: Chip8Fb = [0; CHIP8_FB_W * CHIP8_FB_H];
static mut MEMORY_BUFF: [u8; CHIP8_MEM_SIZE] = [0u8; CHIP8_MEM_SIZE];
static mut EMULATOR: MaybeUninit::<Chip8Emulator> = MaybeUninit::uninit();

#[no_mangle]
pub unsafe extern fn chip8_init() {
    chip8_reset();
}

#[no_mangle]
pub unsafe extern fn chip8_reset() {
    let emu = &mut *EMULATOR.as_mut_ptr();
    *emu = Chip8Emulator::new(EMU_CPU_HZ);
    emu.peripherals_mut().screen.set_inverted_y(false);
    emu.load_rom(&MEMORY_BUFF);
}

#[no_mangle]
pub unsafe extern fn chip8_advance_ms(ms: u32) -> bool {
    let emu = &mut *EMULATOR.as_mut_ptr();
    FRAMEBUFFER.copy_from_slice(emu.framebuffer());
    for x in FRAMEBUFFER.iter_mut() {
        *x |= 0xFF_00_00_00;
    }
    match emu.advance_ms(ms) {
        Ok(()) => true,
        Err(_) => false
    }
}

#[no_mangle]
pub unsafe extern fn chip8_key_down(k: u32) {
    let emu = &mut *EMULATOR.as_mut_ptr();
    emu.peripherals_mut().keypad.key_pressed(k as u8);
}

#[no_mangle]
pub unsafe extern fn chip8_key_up(k: u32) {
    let emu = &mut *EMULATOR.as_mut_ptr();
    emu.peripherals_mut().keypad.key_released(k as u8);
}

#[no_mangle]
pub unsafe extern fn chip8_fb() -> &'static [u32; CHIP8_FB_W * CHIP8_FB_H] {
    &FRAMEBUFFER
}


#[no_mangle]
pub unsafe extern fn chip8_fb_width() -> u32 {
    CHIP8_FB_W as u32
}

#[no_mangle]
pub unsafe extern fn chip8_fb_height() -> u32 {
    CHIP8_FB_H as u32
}

#[no_mangle]
pub unsafe extern fn chip8_memory() -> &'static [u8; CHIP8_MEM_SIZE] {
    &MEMORY_BUFF
}

#[no_mangle]
pub unsafe extern fn chip8_set_cpu_hz(hz: u32) -> i32 {
    let emu = &mut *EMULATOR.as_mut_ptr();
    if hz < CHIP8_PERIPH_HZ {
        return -1;
    }
    EMU_CPU_HZ = hz;
    emu.set_cpu_hz(hz);
    0
}

#[panic_handler]
fn handle_panic(_: &PanicInfo) -> ! {
    loop {}
}
