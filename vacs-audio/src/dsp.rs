pub fn downmix_interleaved_to_mono(
    interleaved: &[f32],
    channels: usize,
    mono: &mut Vec<f32>,
) {
    debug_assert!(channels > 0);
    debug_assert_eq!(interleaved.len() % channels, 0);

    let frames = interleaved.len() / channels;
    mono.clear();
    mono.reserve(frames);
    for frame in interleaved.chunks(channels) {
        mono.push(downmix_frame_to_mono(frame));
    }
}

pub fn upmix_mono_to_interleaved(
    mono: &[f32],
    channels: usize,
    interleaved: &mut [f32],
) {
    debug_assert!(channels > 0);
    debug_assert_eq!(interleaved.len(), mono.len() * channels);

    for (i, &sample) in mono.iter().enumerate() {
        let start = i * channels;
        for ch in 0..channels {
            interleaved[start + ch] = sample;
        }
    }
}

#[inline]
fn downmix_frame_to_mono(frame: &[f32]) -> f32 {
    match frame.len() {
        0 => 0.0f32,
        1 => frame[0],
        2 => {
            let (l, r) = (frame[0], frame[1]);
            if (l - r).abs() < 1e-4 {
                l
            } else {
                (l + r) * 0.5f32
            }
        }
        n => frame.iter().take(n).copied().sum::<f32>() / (n as f32),
    }
}

#[derive(Debug, Clone)]
pub struct LinearResampler {
    /// in_hz / out_hz
    step: f32,
    /// fractional read position into the input stream
    pos: f32,
    /// last sample to allow continuity across callback boundaries
    last: f32,
}

impl LinearResampler {
    pub fn new(from_hz: u32, to_hz: u32) -> Self {
        // We generate `to_hz` samples per second from `from_hz` input samples per second.
        // Each output sample advances input read position by from/to.
        let step = from_hz as f32 / to_hz as f32;
        Self { step, pos: 0.0, last: 0.0 }
    }

    /// Resample mono `input` into `out` (appends). Keep the struct between callbacks to keep continuity.
    pub fn process(&mut self, input: &[f32], out: &mut Vec<f32>) {
        out.clear();
        if input.is_empty() {
            return;
        }
        let mut t = self.pos;
        // We'll sample between "prev" (either last from the previous call or input[i-1]) and input[i]
        let mut prev = self.last;
        let mut idx = 0usize;

        // We want to produce as many output samples as fit within this input chunk.
        // While t < input.len():
        while t < input.len() as f32 {
            let i = t.floor() as usize;
            let frac = t - i as f32;

            // choose current and previous samples for linear interp
            let curr = input[i];
            let a = if i == 0 { prev } else { input[i - 1] };
            let b = curr;

            out.push(a + (b - a) * frac);

            t += self.step;

            // keep prev in sync when we cross sample boundaries
            if i > idx {
                prev = input[i];
                idx = i;
            }
        }

        // Store fractional position and last sample for continuity
        self.pos = t - input.len() as f32;
        self.last = *input.last().unwrap_or(&self.last);
    }
}