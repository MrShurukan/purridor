use core::f32::consts::PI;
use libm::sinf;

/// Helper struct to modulate a value using a sine wave.
/// from and to specify bounds, while period specifies how often it should pulse in seconds
///
/// See [SinePulser::new]
pub struct SinePulser {
    frequency_rad: f32,
    half_diff: f32,
    midpoint: f32,
}

impl SinePulser {
    pub const fn new(from: f32, to: f32, period: f32) -> Self {
        let frequency_rad = if period == 0.0 {
            0.0
        } else {
            (2.0*PI) / period
        };

        let half_diff = (to - from) / 2.0;

        SinePulser {
            frequency_rad,
            half_diff,
            midpoint: from + half_diff,
        }
    }

    pub fn pulse(&self, time: f32) -> f32 {
        self.midpoint + sinf(self.frequency_rad * time) * self.half_diff
    }
}