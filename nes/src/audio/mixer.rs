use super::apu::ApuState;

pub struct Mixer;

impl Mixer {
    pub fn sample(data: ApuState) -> f32 {
        let sq1 = data.square1 as f32;
        let sq2 = data.square2 as f32;

        let square_out = if sq1 == 0.0 && sq2 == 0.0 {
            0.0
        } else {
            95.88 / ((8128.0 / (sq1 + sq2)) + 100.0)
        };

        let t = data.triangle as f32 / 8227.0;
        let n = data.noise as f32 / 12241.0;
        let d = data.dmc as f32 / 22638.0;
        let tnd_out = if t + n + d == 0.0 {
            0.0
        } else {
            159.79 / ((1.0 / (t + n + d)) + 100.0)
        };

        square_out + tnd_out
    }
}
