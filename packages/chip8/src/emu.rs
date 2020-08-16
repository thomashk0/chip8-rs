use crate::Chip8Cpu;
use crate::screen::Screen;
use crate::keypad::Keypad;
use crate::cpu::CpuError;

pub const CHIP8_PERIPH_HZ : u32 = 60;

const SPRITE_DATA: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,   // 0
    0x20, 0x60, 0x20, 0x20, 0x70,   // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0,   // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0,   // 3
    0x90, 0x90, 0xF0, 0x10, 0x10,   // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0,   // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0,   // 6
    0xF0, 0x10, 0x20, 0x40, 0x40,   // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0,   // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0,   // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90,   // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0,   // B
    0xF0, 0x80, 0x80, 0x80, 0xF0,   // C
    0xE0, 0x90, 0x90, 0x90, 0xE0,   // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0,   // E
    0xF0, 0x80, 0xF0, 0x80, 0x80    // F
];

#[derive(Clone)]
pub struct Chip8Peripherals {
    pub memory: [u8; 4096],
    pub screen: Screen,
    pub keypad: Keypad,
    pub delay_timer: u16,
    pub sound_timer: u16,
}

impl Chip8Peripherals {
    pub fn new() -> Self {
        let mut memory = [0u8; 4096];
        // Place sprite data at the begining of memory
        for (w, r) in SPRITE_DATA.iter().zip(memory.iter_mut()) {
            *r = *w;
        }
        let mut screen = Screen::new();
        screen.clear(0);
        Chip8Peripherals {
            memory,
            screen,
            keypad: Keypad::new(),
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    pub fn tick(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
}

#[derive(Clone)]
pub struct Chip8Emulator {
    cpu_hz: u32,
    periph_hz: u32,
    cpu: Chip8Cpu,
    periph: Chip8Peripherals,
    sim_ms: u32,
}

impl Chip8Emulator {
    pub fn new(cpu_hz: u32) -> Self {
        Chip8Emulator {
            cpu_hz,
            cpu: Chip8Cpu::new(0x200),
            periph_hz: 60,
            periph: Chip8Peripherals::new(),
            sim_ms: 0,
        }
    }

    pub fn peripherals(&self) -> &Chip8Peripherals {
        &self.periph
    }

    pub fn peripherals_mut(&mut self) -> &mut Chip8Peripherals {
        &mut self.periph
    }

    pub fn set_cpu_hz(&mut self, hz: u32) {
        self.cpu_hz = hz
    }

    pub fn set_cpu_rng_seed(&mut self, seed: u64) {
        self.cpu.set_rng_seed(seed);
    }

    pub fn framebuffer_dims(&self) -> (u32, u32) {
        let screen = &self.periph.screen;
        (screen.width(), screen.height())
    }

    pub fn framebuffer(&self) -> &[u32] {
        self.periph.screen.data()
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        let n = data.len();
        self.periph.memory[0x200..0x200 + n].copy_from_slice(data);
    }

    pub fn advance_ms(&mut self, ms: u32) -> Result<(), CpuError> {
        let periph_hz = self.periph_hz;
        let cpu_hz = self.cpu_hz;
        let mut t = self.sim_ms;
        {
            let Chip8Emulator { periph, cpu, .. } = self;
            let cpu_steps = cpu_hz * (ms as u32) / 1000;
            for _ in 0..cpu_steps {
                cpu.tick(periph)?;
                t += periph_hz;
                if t >= self.cpu_hz {
                    periph.tick();
                    t -= self.cpu_hz;
                }
            }
        }
        self.sim_ms = t;
        Ok(())
    }
}