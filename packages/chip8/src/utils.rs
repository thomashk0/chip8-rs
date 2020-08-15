use core::num::Wrapping;

type W64 = Wrapping<u64>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pcg32 {
    // Private fields
    state: u64,
    inc: u64,
}

impl Default for Pcg32 {
    fn default() -> Self {
        Pcg32 { state: 0, inc: 0 }
    }
}

impl Pcg32 {
    pub fn reset(&mut self, state: u64, inc: u64) {
        self.state = 0;
        self.inc = (Wrapping(inc) << 1).0 | 1;
        self.generate();
        self.state = self.state.wrapping_add(state);
        self.generate();
    }

    pub fn generate(&mut self) -> u32 {
        const DEFAULT_MULT: W64 = Wrapping(0x5851_f42d_4c95_7f2d);
        let old_state = Wrapping(self.state);
        self.state = (old_state * DEFAULT_MULT + Wrapping(self.inc)).0;
        let xor_shifted = ((old_state >> 18) ^ old_state) >> 27;
        let rot = old_state >> 59;
        let shift = (!rot + Wrapping(1)) & Wrapping(31);
        let result = (xor_shifted >> (rot.0 as usize)) | (xor_shifted << (shift.0 as usize));
        result.0 as u32
    }
}