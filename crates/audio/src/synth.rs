//! Audio synthesis: software oscillators, envelopes, and drum sounds.
//!
//! All sources implement [`rodio::Source`] so they can be mixed together.

use std::time::Duration;

use rodio::Source;

// ── Waveform ────────────────────────────────────────────────────

/// Waveform shape used by the oscillator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
}

// ── Oscillator ──────────────────────────────────────────────────

/// A simple oscillator producing a tone at a given frequency.
pub struct Oscillator {
    waveform: Waveform,
    freq: f32,
    sample_rate: u32,
    sample_idx: u64,
    duration_samples: u64,
    // ADSR envelope
    attack_samples: u64,
    decay_samples: u64,
    sustain_level: f32,
    release_samples: u64,
    amplitude: f32,
}

impl Oscillator {
    /// Create a new oscillator.
    ///
    /// - `waveform` — shape of the wave
    /// - `freq` — frequency in Hz
    /// - `duration` — total duration
    /// - `amplitude` — 0.0..1.0
    pub fn new(waveform: Waveform, freq: f32, duration: Duration, amplitude: f32) -> Self {
        let sample_rate = 44100;
        let dur_samples = (duration.as_secs_f64() * sample_rate as f64) as u64;
        let attack = (0.01 * sample_rate as f64) as u64; // 10ms
        let decay = (0.05 * sample_rate as f64) as u64; // 50ms
        let release = (0.06 * sample_rate as f64) as u64; // 60ms

        Self {
            waveform,
            freq,
            sample_rate,
            sample_idx: 0,
            duration_samples: dur_samples,
            attack_samples: attack,
            decay_samples: decay,
            sustain_level: 0.6,
            release_samples: release,
            amplitude,
        }
    }

    /// Apply a custom ADSR to this oscillator.
    pub fn with_adsr(
        mut self,
        attack_ms: f64,
        decay_ms: f64,
        sustain: f32,
        release_ms: f64,
    ) -> Self {
        self.attack_samples = (attack_ms / 1000.0 * self.sample_rate as f64) as u64;
        self.decay_samples = (decay_ms / 1000.0 * self.sample_rate as f64) as u64;
        self.sustain_level = sustain;
        self.release_samples = (release_ms / 1000.0 * self.sample_rate as f64) as u64;
        self
    }

    fn phase(&self) -> f64 {
        let t = self.sample_idx as f64 / self.sample_rate as f64;
        (t * self.freq as f64).fract()
    }

    fn wave_sample(&self) -> f32 {
        let phase = self.phase();
        match self.waveform {
            Waveform::Sine => (phase * 2.0 * std::f64::consts::PI).sin() as f32,
            Waveform::Square => {
                if phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::Sawtooth => (2.0 * phase - 1.0) as f32,
            Waveform::Triangle => {
                let v = if phase < 0.25 {
                    phase * 4.0
                } else if phase < 0.75 {
                    2.0 - phase * 4.0
                } else {
                    phase * 4.0 - 4.0
                };
                v as f32
            }
        }
    }

    fn envelope(&self) -> f32 {
        let i = self.sample_idx;
        let note_off = self.duration_samples.saturating_sub(self.release_samples);

        if i < self.attack_samples {
            // Attack
            i as f32 / self.attack_samples.max(1) as f32
        } else if i < self.attack_samples + self.decay_samples {
            // Decay
            let decay_pos = (i - self.attack_samples) as f32 / self.decay_samples.max(1) as f32;
            1.0 - (1.0 - self.sustain_level) * decay_pos
        } else if i < note_off {
            // Sustain
            self.sustain_level
        } else {
            // Release
            let release_pos = (i - note_off) as f32 / self.release_samples.max(1) as f32;
            self.sustain_level * (1.0 - release_pos).max(0.0)
        }
    }
}

impl Iterator for Oscillator {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.sample_idx >= self.duration_samples {
            return None;
        }
        let sample = self.wave_sample() * self.envelope() * self.amplitude;
        self.sample_idx += 1;
        Some(sample)
    }
}

impl Source for Oscillator {
    fn current_frame_len(&self) -> Option<usize> {
        let remaining = self.duration_samples.saturating_sub(self.sample_idx);
        Some(remaining as usize)
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f64(
            self.duration_samples as f64 / self.sample_rate as f64,
        ))
    }
}

// ── Noise Burst (drums) ────────────────────────────────────────

/// A short noise burst with an exponential decay — used for hi-hats, snares etc.
pub struct NoiseBurst {
    sample_rate: u32,
    sample_idx: u64,
    duration_samples: u64,
    amplitude: f32,
    rng_state: u32,
}

impl NoiseBurst {
    pub fn new(duration: Duration, amplitude: f32) -> Self {
        let sample_rate = 44100;
        Self {
            sample_rate,
            sample_idx: 0,
            duration_samples: (duration.as_secs_f64() * sample_rate as f64) as u64,
            amplitude,
            rng_state: 0xDEAD_BEEF,
        }
    }

    fn next_noise(&mut self) -> f32 {
        // Simple xorshift
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        (self.rng_state as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

impl Iterator for NoiseBurst {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.sample_idx >= self.duration_samples {
            return None;
        }
        let progress = self.sample_idx as f32 / self.duration_samples.max(1) as f32;
        let envelope = (-progress * 8.0).exp(); // fast exponential decay
        let sample = self.next_noise() * envelope * self.amplitude;
        self.sample_idx += 1;
        Some(sample)
    }
}

impl Source for NoiseBurst {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.duration_samples.saturating_sub(self.sample_idx) as usize)
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f64(
            self.duration_samples as f64 / self.sample_rate as f64,
        ))
    }
}

// ── Kick Drum ───────────────────────────────────────────────────

/// A synthesised kick drum: a sine sweep from high to low with exponential decay.
pub struct KickDrum {
    sample_rate: u32,
    sample_idx: u64,
    duration_samples: u64,
    amplitude: f32,
}

impl KickDrum {
    pub fn new(amplitude: f32) -> Self {
        let sample_rate = 44100;
        let duration = Duration::from_millis(200);
        Self {
            sample_rate,
            sample_idx: 0,
            duration_samples: (duration.as_secs_f64() * sample_rate as f64) as u64,
            amplitude,
        }
    }
}

impl Iterator for KickDrum {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.sample_idx >= self.duration_samples {
            return None;
        }
        let t = self.sample_idx as f64 / self.sample_rate as f64;
        // Frequency sweep: 150 Hz → 40 Hz
        let freq = 40.0 + 110.0 * (-t * 30.0).exp();
        let envelope = (-t * 15.0).exp();
        let sample = (t * freq * 2.0 * std::f64::consts::PI).sin() * envelope;
        self.sample_idx += 1;
        Some(sample as f32 * self.amplitude)
    }
}

impl Source for KickDrum {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.duration_samples.saturating_sub(self.sample_idx) as usize)
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f64(
            self.duration_samples as f64 / self.sample_rate as f64,
        ))
    }
}

// ── Hihat ───────────────────────────────────────────────────────

/// A short metallic hihat — filtered noise burst.
pub struct Hihat {
    inner: NoiseBurst,
}

impl Hihat {
    pub fn new(amplitude: f32) -> Self {
        Self {
            inner: NoiseBurst::new(Duration::from_millis(60), amplitude),
        }
    }
}

impl Iterator for Hihat {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        // Simulate highpass by emphasising noise transients
        self.inner.next().map(|s| s * 0.7)
    }
}

impl Source for Hihat {
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}

// ── Snare ───────────────────────────────────────────────────────

/// Synthesised snare: noise + low sine transient.
pub struct Snare {
    sample_rate: u32,
    sample_idx: u64,
    duration_samples: u64,
    amplitude: f32,
    rng_state: u32,
}

impl Snare {
    pub fn new(amplitude: f32) -> Self {
        let sample_rate = 44100;
        Self {
            sample_rate,
            sample_idx: 0,
            duration_samples: (0.15 * sample_rate as f64) as u64,
            amplitude,
            rng_state: 0xCAFE_BABE,
        }
    }

    fn noise(&mut self) -> f32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        (self.rng_state as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

impl Iterator for Snare {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.sample_idx >= self.duration_samples {
            return None;
        }
        let t = self.sample_idx as f64 / self.sample_rate as f64;
        let envelope = (-t * 20.0).exp();
        let body = (t * 180.0 * 2.0 * std::f64::consts::PI).sin() as f32 * 0.5;
        let noise = self.noise() * 0.5;
        self.sample_idx += 1;
        Some((body + noise) * envelope as f32 * self.amplitude)
    }
}

impl Source for Snare {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.duration_samples.saturating_sub(self.sample_idx) as usize)
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f64(
            self.duration_samples as f64 / self.sample_rate as f64,
        ))
    }
}
