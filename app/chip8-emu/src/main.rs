use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io::Read;

use std::time::SystemTime;

use sdl2::audio::{AudioCallback, AudioSpecDesired, AudioSpec};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use structopt::StructOpt;

use chip8::{Chip8Emulator, Insn};
use gl_scene::Scene;
use std::sync::{Arc, RwLock};

mod gl_scene;

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        let enabled = *self.enabled.read().unwrap();
        for x in out.iter_mut() {
            if !enabled {
                *x = 0.0;
                continue;
            }
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}


struct SquareWave {
    enabled: Arc<RwLock<bool>>,
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl SquareWave {
    pub fn new(spec: &AudioSpec, enabled: Arc<RwLock<bool>>) -> Self {
        SquareWave {
            enabled,
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.25,
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "chip8emu", about = "Rust-powered Chip 8 Emulator (SDL2-OpenGL GUI)")]
pub struct CliOpts {
    #[structopt(short = "h", long = "cpuhz", default_value = "500")]
    emu_hz: u32,

    #[structopt(name = "FILE.ch8")]
    rom_path: String,
}

fn run_emulator(emu: &mut Chip8Emulator) -> Result<(), Box<dyn std::error::Error>> {
    let keymap: HashMap<Keycode, u8> = [
        (Keycode::Kp1, 1),
        (Keycode::Kp2, 2),
        (Keycode::Kp3, 3),
        (Keycode::C, 0xC),
        (Keycode::Kp4, 4),
        (Keycode::Kp5, 5),
        (Keycode::Kp6, 6),
        (Keycode::D, 0xD),
        (Keycode::Kp7, 7),
        (Keycode::Kp8, 8),
        (Keycode::Kp9, 9),
        (Keycode::E, 0xE),
        (Keycode::A, 0xA),
        (Keycode::Kp0, 0),
        (Keycode::B, 0xB),
        (Keycode::F, 0xF)]
        .iter().cloned().collect();
    const WIN_W: u32 = 512;
    const WIN_H: u32 = 256;

    let (emu_w, emu_h) = emu.peripherals().screen.dims();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("rust-sdl2 demo", WIN_W, WIN_H)
        .position_centered()
        .opengl()
        .resizable()
        .build()
        .unwrap();
    let _ctx = window.gl_create_context().unwrap();
    // Uncomment the following to not wait for vertical refresh
    //    video_subsystem
    //        .gl_set_swap_interval(SwapInterval::Immediate)
    //        .unwrap();
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);

    let buzzer_active = Arc::new(RwLock::new(false));
    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),  // mono
        samples: None,       // default sample size
    };
    let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        SquareWave::new(&spec, buzzer_active.clone())
    }).unwrap();
    device.resume();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let scene = Scene::new();

    let mut timer = SystemTime::now();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(k), .. } => {
                    if k == Keycode::Escape {
                        break 'running;
                    }
                    if let Some(k_num) = keymap.get(&k).cloned() {
                        emu.peripherals_mut().keypad.key_pressed(k_num);
                    } else {
                        eprintln!("note: unmapped keycode (down): {:?}", k);
                    }
                }
                Event::KeyUp { keycode: Some(k), .. } => {
                    if let Some(k_num) = keymap.get(&k).cloned() {
                        emu.peripherals_mut().keypad.key_released(k_num);
                    } else {
                        eprintln!("note: unmapped keycode (up): {:?}", k);
                    }
                }
                _ => {}
            }
        }

        let ms = timer.elapsed().unwrap().as_millis();
        timer = SystemTime::now();
        match emu.advance_ms(ms as u32) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("error: CPU crashed during emulation: {:?}", e);
            }
        }
        // Note: the buzzer is updated at 60Hz at best...
        {
            let mut buzzer_ptr = buzzer_active.write().unwrap();
            *buzzer_ptr = emu.peripherals().sound_timer > 0;
        }

        unsafe {
            gl::ClearColor(0., 0., 0., 1.);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            scene.render(emu.framebuffer(), emu_w, emu_h);
            gl::BindVertexArray(0);
        }
        window.gl_swap_window();
    }
    Ok(())
}

pub fn run_app(opts: &CliOpts) -> Result<i32, Box<dyn std::error::Error>> {
    // let args: Vec<String> = env::args().collect();
    // if args.len() < 1 {
    //     eprintln!("No ROM given :(\nUSAGE: {} ROM.ch8", args[0]);
    //     return Ok(1);
    // }
    let mut f = File::open(&opts.rom_path)?;
    let mut buffer = Vec::new();
    // read the whole file
    f.read_to_end(&mut buffer)?;
    println!("{}", buffer.len());
    for (i, bs) in buffer.chunks_exact(2).enumerate() {
        let insn = u16::from_be_bytes(bs.try_into().unwrap());
        print!("{:4x}: {:04x} ", 2 * i, insn);
        if let Some(w) = Insn::decode(insn) {
            println!("{:?}", w);
        } else {
            println!("<INVALID>")
        }
    }
    let mut emulator = Chip8Emulator::new(opts.emu_hz);
    emulator.load_rom(&buffer);
    // TODO: read from command line
    emulator.set_cpu_rng_seed(0x1234_56789);
    run_emulator(&mut emulator)?;
    Ok(0)
}

fn main() {
    let opt = CliOpts::from_args();
    println!("Chip 8 Emulator\n2020, Thomas Hiscock\n");
    match run_app(&opt) {
        Ok(x) => {
            std::process::exit(x)
        }
        Err(e) => {
            eprintln!("error: an unexpected error occured in the application {:?}.\nPlease contact the developpers.", e);
            std::process::exit(1)
        }
    }
}
