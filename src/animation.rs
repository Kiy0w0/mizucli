pub struct BadAppleState {
    pub frame: usize,
    accum: f64,
}

impl BadAppleState {
    pub fn new() -> Self {
        Self { frame: 0, accum: 0.0 }
    }

    pub fn tick(&mut self, delta: f64) {
        self.accum += delta;
        let frame_dur = 1.0 / 30.0;
        while self.accum >= frame_dur {
            self.accum -= frame_dur;
            self.frame = (self.frame + 1) % crate::bad_apple_frames::FRAME_COUNT;
        }
    }

    pub fn decode_current(&self) -> Vec<u8> {
        let rle = crate::bad_apple_frames::FRAMES[self.frame];
        let total = crate::bad_apple_frames::W as usize * crate::bad_apple_frames::H as usize;
        let mut buf = Vec::with_capacity(total);
        for &(count, val) in rle {
            for _ in 0..count {
                buf.push(val);
            }
        }
        buf
    }
}
