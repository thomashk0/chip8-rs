//
// # Usefull references
//
// * http://mattmik.com/files/chip8/mastering/chip8.html
// * https://hackaday.io/project/19121-andxor-dc25-badge/log/53223-chip8-schip-game-emulation
// * https://www.onlinegdb.com/ryyYBu2m8
// * https://blog.scottlogic.com/2017/12/13/chip8-emulator-webassembly-rust.html
use core::convert::TryInto;
use core::ops::Shl;
use crate::{Pcg32, Chip8Peripherals};

type Word = u8;
type Addr = u16;
type Reg = u8;

//
// Utils
//

fn get_b0(x: u16) -> u8 {
    x as u8
}

fn get_hb0(x: u16) -> u16 {
    x & 0xF
}

fn get_hb1(x: u16) -> u16 {
    (x >> 4) & 0xF
}

fn get_hb2(x: u16) -> u16 {
    (x >> 8) & 0xF
}

fn get_hb3(x: u16) -> u16 {
    (x >> 12) & 0xF
}

fn bool_to_bit(b: bool) -> u8 {
    if b { 1 } else { 0 }
}

const SCREEN_W: usize = 64;
const SCREEN_H: usize = 32;

//
// Chip8 API
//

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Insn {
    Cls,
    Ret,
    Jump(Addr),
    JumpV0(Addr),
    Call(Addr),
    SkipEqI(Reg, Word),
    SkipNeqI(Reg, Word),
    SkipEq(Reg, Reg),
    SkipNeq(Reg, Reg),
    LoadI(Reg, Word),
    AddI(Reg, Word),
    Move(Reg, Reg),
    Or(Reg, Reg),
    And(Reg, Reg),
    Xor(Reg, Reg),
    Add(Reg, Reg),
    Sub(Reg, Reg),
    Shr(Reg, Reg),
    SubN(Reg, Reg),
    Shl(Reg, Reg),
    LoadA(u16),
    AddA(Reg),
    RndAnd(Reg, Word),
    DrawSprite(Reg, Reg, Word),
    SkipKeyPressed(Reg),
    SkipKeyNPressed(Reg),
    LoadTimer(Reg),
    WaitForKey(Reg),
    SetDelayTimer(Reg),
    SetSoundTimer(Reg),
    SpriteLoc(Reg),
    StoreBCD(Reg),
    StoreRegs(Reg),
    LoadRegs(Reg),
}


impl Insn {
    /// Extract opcode and operands from an instruction
    pub fn decode(insn: u16) -> Option<Self> {
        // Design note: considering the size of the ISA, having a table-based
        // decoder seems completely overkill!
        let rx = get_hb2(insn) as Reg;
        let ry = get_hb1(insn) as Reg;
        match get_hb3(insn) {
            0 => {
                if insn == 0x00E0 {
                    Some(Insn::Cls)
                } else if insn == 0x00EE {
                    Some(Insn::Ret)
                } else {
                    None
                }
            }
            1 => {
                Some(Insn::Jump(insn & 0xFFF))
            }
            2 => {
                Some(Insn::Call(insn & 0xFFF))
            }
            3 => {
                Some(Insn::SkipEqI(rx, get_b0(insn)))
            }
            4 => {
                Some(Insn::SkipNeqI(rx, get_b0(insn)))
            }
            5 => {
                if get_hb0(insn) != 0 {
                    return None;
                }
                Some(Insn::SkipEq(rx, ry))
            }
            6 => {
                Some(Insn::LoadI(rx, get_b0(insn)))
            }
            7 => {
                Some(Insn::AddI(rx, get_b0(insn)))
            }
            8 => {
                let rx = rx;
                let ry = ry;
                match get_hb0(insn) {
                    0 => Some(Insn::Move(rx, ry)),
                    1 => Some(Insn::Or(rx, ry)),
                    2 => Some(Insn::And(rx, ry)),
                    3 => Some(Insn::Xor(rx, ry)),
                    4 => Some(Insn::Add(rx, ry)),
                    5 => Some(Insn::Sub(rx, ry)),
                    6 => Some(Insn::Shr(rx, ry)),
                    7 => Some(Insn::SubN(rx, ry)),
                    0xE => Some(Insn::Shl(rx, ry)),
                    _ => None
                }
            }
            9 => {
                if get_hb0(insn) != 0 {
                    return None;
                }
                Some(Insn::SkipNeq(rx, ry))
            }
            0xA => {
                Some(Insn::LoadA(insn & 0xFFF))
            }
            0xB => {
                Some(Insn::JumpV0(insn & 0xFFF))
            }
            0xC => {
                Some(Insn::RndAnd(rx, get_b0(insn)))
            }
            0xD => {
                Some(Insn::DrawSprite(rx, ry, get_hb0(insn) as u8))
            }
            0xE => {
                match get_b0(insn) {
                    0x9E => Some(Insn::SkipKeyPressed(rx)),
                    0xA1 => Some(Insn::SkipKeyNPressed(rx)),
                    _ => None
                }
            }
            0xF => {
                match get_b0(insn) {
                    0x07 => Some(Insn::LoadTimer(rx)),
                    0x0A => Some(Insn::WaitForKey(rx)),
                    0x15 => Some(Insn::SetDelayTimer(rx)),
                    0x18 => Some(Insn::SetSoundTimer(rx)),
                    0x1E => Some(Insn::AddA(rx)),
                    0x29 => Some(Insn::SpriteLoc(rx)),
                    0x33 => Some(Insn::StoreBCD(rx)),
                    0x55 => Some(Insn::StoreRegs(rx)),
                    0x65 => Some(Insn::LoadRegs(rx)),
                    _ => None
                }
            }
            _ => None
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CpuStatus {
    Running,
    WaitEvent,
    Halted,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Chip8Cpu {
    status: CpuStatus,
    gpr: [Word; 16],
    reg_i: u16,
    stack: [u16; 16],
    pc: Addr,
    sp: Addr,
    rng: Pcg32,
    cycles: u64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CpuError {
    InvalidInstruction,
    NotImplemented,
    MemoryError,
    PopEmptyStack,
    StackOverflow,
    InvalidSprite,
}

impl Chip8Cpu {
    pub fn new(boot_addr: Addr) -> Self {
        Chip8Cpu {
            status: CpuStatus::Running,
            gpr: [0; 16],
            pc: boot_addr,
            reg_i: 0,
            sp: 0,
            stack: [0; 16],
            rng: Pcg32::new(),
            cycles: 0,
        }
    }

    pub fn read_gpr(&self, r: Reg) -> Word {
        assert!(r < 16);
        self.gpr[r as usize]
    }

    pub fn write_gpr(&mut self, r: Reg, value: Word) {
        assert!(r < 16);
        self.gpr[r as usize] = value;
    }

    pub fn write_vf(&mut self, value: Word) {
        self.write_gpr(0xF, value);
    }

    pub fn write_vf_flag(&mut self, value: bool) {
        self.write_gpr(0xF, bool_to_bit(value));
    }

    pub fn set_rng_seed(&mut self, seed: u64) {
        self.rng.reset(seed, 42);
    }

    pub fn exec_insn(&mut self, insn: Insn, periph: &mut Chip8Peripherals) -> Result<Option<Addr>, CpuError> {
        match insn {
            Insn::Cls => periph.screen.clear(0),
            Insn::Ret => {
                if self.sp == 0 {
                    return Err(CpuError::PopEmptyStack);
                }
                self.sp -= 1;
                return Ok(Some(self.stack[self.sp as usize]));
            }
            Insn::Jump(target) => {
                return Ok(Some(target));
            }
            Insn::JumpV0(target) => {
                let dst = self.read_gpr(0) as u16 + target;
                return Ok(Some(dst));
            }
            Insn::Call(target) => {
                if self.sp == 16 {
                    return Err(CpuError::StackOverflow);
                }
                self.stack[self.sp as usize] = self.pc + 2;
                self.sp += 1;
                return Ok(Some(target));
            }
            Insn::SkipEqI(r, value) => {
                if self.read_gpr(r) == value {
                    return Ok(Some(self.pc + 4));
                }
            }
            Insn::SkipNeqI(r, value) => {
                if self.read_gpr(r) != value {
                    return Ok(Some(self.pc + 4));
                }
            }
            Insn::SkipEq(rx, ry) => {
                if self.read_gpr(rx) == self.read_gpr(ry) {
                    return Ok(Some(self.pc + 4));
                }
            }
            Insn::SkipNeq(rx, ry) => {
                if self.read_gpr(rx) != self.read_gpr(ry) {
                    return Ok(Some(self.pc + 4));
                }
            }
            Insn::LoadI(r, v) => self.write_gpr(r, v),
            Insn::AddI(rx, v) => self.write_gpr(rx, self.read_gpr(rx).wrapping_add(v)),
            Insn::Move(rx, ry) => self.write_gpr(rx, self.read_gpr(ry)),
            Insn::Or(rx, ry) => self.write_gpr(rx, self.read_gpr(rx) | self.read_gpr(ry)),
            Insn::And(rx, ry) => self.write_gpr(rx, self.read_gpr(rx) & self.read_gpr(ry)),
            Insn::Xor(rx, ry) => self.write_gpr(rx, self.read_gpr(rx) ^ self.read_gpr(ry)),
            Insn::Add(rx, ry) => {
                let r = self.read_gpr(rx) as u16 + self.read_gpr(ry) as u16;
                self.write_vf_flag(r >= 256);
                self.write_gpr(rx, r as u8);
            }
            Insn::Sub(rx, ry) => {
                let x = self.read_gpr(rx);
                let y = self.read_gpr(ry);
                self.write_vf_flag(x > y);
                self.write_gpr(rx, x.wrapping_sub(y));
            }
            Insn::SubN(rx, ry) => {
                let x = self.read_gpr(rx);
                let y = self.read_gpr(ry);
                self.write_vf_flag(y > x);
                self.write_gpr(rx, y.wrapping_sub(x));
            }
            Insn::Shr(r, _) => {
                let x = self.gpr[r as usize];
                self.write_vf(x & 1);
                self.write_gpr(r, x >> 1);
            }
            Insn::Shl(r, _) => {
                let x = self.read_gpr(r);
                self.write_vf(x >> 7);
                self.write_gpr(r, x.shl(1));
            }
            Insn::LoadA(value) => self.reg_i = value,
            Insn::AddA(r) => {
                self.reg_i = self.reg_i.wrapping_add(self.read_gpr(r) as u16);
            }
            Insn::RndAnd(r, value) => {
                let rnd = self.rng.next() as u8;
                self.write_gpr(r, rnd & value);
            }
            Insn::DrawSprite(rx, ry, n) => {
                let x = self.read_gpr(rx);
                let y = self.read_gpr(ry);
                let base = self.reg_i as usize;
                let mut erased = false;
                for dy in 0u8..n {
                    let mut w = periph.memory[base + dy as usize];
                    // Design note: lowest bit is the 8-th pixel. Thus, we
                    // reverse the iteration order.
                    for dx in (0u8..8).rev() {
                        let coords = (((x + dx) % SCREEN_W as u8) as i32, ((y + dy) % SCREEN_H as u8) as i32);
                        let px_value = 0xFFFF_FFFF * (w & 1) as u32;
                        w >>= 1;
                        erased |= periph.screen.xor_pixel(coords, px_value as u32);
                    }
                }
                self.write_vf_flag(erased);
            }
            Insn::SkipKeyPressed(rx) => {
                let key = self.read_gpr(rx);
                if periph.keypad.key_state(key) == 1 {
                    return Ok(Some(self.pc + 4));
                }
            }
            Insn::SkipKeyNPressed(rx) => {
                let key = self.read_gpr(rx);
                if periph.keypad.key_state(key) == 0 {
                    return Ok(Some(self.pc + 4));
                }
            }
            Insn::LoadTimer(rx) => self.write_gpr(rx, periph.delay_timer as u8),
            Insn::WaitForKey(rx) => {
                if let Some(key) = periph.keypad.first_key_pressed() {
                    self.write_gpr(rx, key);
                    self.status = CpuStatus::Running;
                } else {
                    // Stay at the same instruction until resolved.
                    self.status = CpuStatus::WaitEvent;
                    return Ok(Some(self.pc));
                }
            }
            Insn::SetDelayTimer(rx) => periph.delay_timer = self.read_gpr(rx) as u16,
            Insn::SetSoundTimer(rx) => periph.sound_timer = self.read_gpr(rx) as u16,
            Insn::SpriteLoc(rx) => {
                // TODO: has to be checked...
                let x = self.read_gpr(rx);
                if x >= 16 {
                    return Err(CpuError::InvalidSprite);
                }
                self.reg_i = 5 * x as u16;
            }
            Insn::StoreBCD(rx) => {
                let mut x = self.read_gpr(rx);
                for i in (0..3).rev() {
                    periph.memory[self.reg_i as usize + i] = x % 10;
                    x /= 10;
                }
            }
            Insn::StoreRegs(n) => {
                for i in 0..=(n as usize) {
                    periph.memory[self.reg_i as usize + i] = self.gpr[i];
                }
            }
            Insn::LoadRegs(n) => {
                for i in 0..=(n as usize) {
                    self.gpr[i] = periph.memory[self.reg_i as usize + i];
                }
            }
        }
        return Ok(None);
    }

    pub fn tick(&mut self, periph: &mut Chip8Peripherals) -> Result<(), CpuError> {
        self.cycles += 1;
        let pc = self.pc as usize;
        if pc > 0x4096 {
            return Err(CpuError::MemoryError);
        }
        let insn_raw = u16::from_be_bytes(periph.memory[pc..pc + 2].try_into().unwrap());
        let insn = Insn::decode(insn_raw).ok_or(CpuError::InvalidInstruction)?;

        // Debug: instruction tracing
        // println!("{:4x}> {:4x}> {:?}", pc, insn_raw, insn);

        let r = self.exec_insn(insn, periph)?;
        match r {
            Some(new_pc) => self.pc = new_pc,
            None => self.pc += 2
        }
        Ok(())
    }
}