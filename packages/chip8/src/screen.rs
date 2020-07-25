type Point2i = (i32, i32);

pub const CHIP8_FB_W: usize = 64;
pub const CHIP8_FB_H: usize = 32;

pub type Chip8Fb = [u32; CHIP8_FB_W * CHIP8_FB_H];

#[derive(Clone)]
pub struct Screen {
    inverted_y: bool,
    fb: Chip8Fb,
    width: u32,
    height: u32,
}

impl Screen {
    pub fn new() -> Self {
        Screen {
            inverted_y: true,
            fb: [0; CHIP8_FB_W * CHIP8_FB_H],
            width: CHIP8_FB_W as u32,
            height: CHIP8_FB_H as u32,
        }
    }

    pub fn set_inverted_y(&mut self, b: bool) {
        self.inverted_y = b;
    }

    fn px_index(&self, coords: Point2i) -> usize {
        let h = self.height - 1;
        let y = if self.inverted_y { h - coords.1 as u32 } else { coords.1 as u32 };
        let k = y * self.width + (coords.0 as u32);
        k as usize
    }

    pub fn data(&self) -> &[u32] {
        &self.fb
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn set_pixel(&mut self, coords: Point2i, value: u32) {
        let k = (coords.1 as u32) * self.width + coords.0 as u32;
        self.fb[k as usize] = value;
    }

    pub fn xor_pixel(&mut self, coords: Point2i, value: u32) -> bool {
        let k = self.px_index(coords);
        let old_px = self.fb[k as usize];
        self.fb[k] ^= value;
        old_px != 0 && value != 0
    }

    pub fn clear(&mut self, color: u32) {
        self.fb.iter_mut().for_each(|x| *x = color);
    }

    #[cfg(feature = "std")]
    // NOTE: doesn't check points are in bounds...
    pub fn draw_vline(&mut self, x: f32, y0: f32, y1: f32, color: u32) {
        let y0 = y0.floor() as u32;
        let y1 = y1.floor() as u32;
        let y_min = core::cmp::min(y0, y1);
        let y_max = core::cmp::max(y0, y1);
        for y in y_min..y_max {
            let k = y * self.width + x.floor() as u32;
            self.fb[k as usize] = color;
        }
    }

    #[cfg(feature = "std")]
    pub fn draw_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: u32) {
        if x0 == x1 {
            self.draw_vline(x0, y0, y1, color);
            return;
        }
        let dx = x1 - x0;
        let dy = y1 - y0;
        let step = dx.abs().max(dy.abs());
        let mut x = x0;
        let mut y = y0;
        for _ in 0..(step.floor() as u32) {
            x += dx / step;
            y += dy / step;
            let k = (y.floor() as u32) * self.width + x.floor() as u32;
            self.fb[k as usize] = color;
        }
    }
}
