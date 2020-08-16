#[derive(Debug, Copy, Clone)]
pub struct Keypad {
    keystate: u32
}

impl Keypad {
    pub fn new() -> Self {
        Keypad {
            keystate: 0
        }
    }

    pub fn key_pressed(&mut self, key: u8) {
        self.keystate |= 1 << key as u32;
    }

    pub fn key_released(&mut self, key: u8) {
        self.keystate &= !(1 << key as u32);
    }

    /// Reset the keystate (all keys unpressed)
    pub fn clear(&mut self) { self.keystate = 0; }

    pub fn key_state(&self, key: u8) -> u8 {
        (self.keystate >> key as u32) as u8 & 1
    }

    pub fn first_key_pressed(&self) -> Option<u8> {
        if self.keystate == 0 {
            return None;
        }
        for i in 0..16 {
            if (self.keystate >> i as u32) & 1 != 0 {
                return Some(i);
            }
        }
        None
    }
}