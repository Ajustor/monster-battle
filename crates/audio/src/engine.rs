//! Audio engine — plays tracks and SFX through `rodio`.
//!
//! The engine runs on a background thread. The public API is fully thread-safe
//! and designed to be called from any async/TUI context.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use rodio::{OutputStream, OutputStreamHandle, Sink, Source};

use crate::pattern::Event;
use crate::synth::{Hihat, KickDrum, Oscillator, Snare};
use crate::tracks::{self, Sfx, Track, Voice};

// ── Public API ──────────────────────────────────────────────────

/// The audio engine.
///
/// Cheap to clone — all clones share the same inner state.
#[derive(Clone)]
pub struct AudioEngine {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    music_sink: Sink,
    sfx_sinks: Vec<Sink>,
    current_track: Option<&'static str>,
    music_volume: f32,
    sfx_volume: f32,
    muted: bool,
}

impl AudioEngine {
    /// Create a new audio engine.
    ///
    /// Returns `None` if no audio output device is available (e.g. CI, headless).
    pub fn try_new() -> Option<Self> {
        let (stream, handle) = OutputStream::try_default().ok()?;
        let music_sink = Sink::try_new(&handle).ok()?;
        music_sink.set_volume(0.5);

        Some(Self {
            inner: Arc::new(Mutex::new(Inner {
                _stream: stream,
                stream_handle: handle,
                music_sink,
                sfx_sinks: Vec::new(),
                current_track: None,
                music_volume: 0.5,
                sfx_volume: 0.7,
                muted: false,
            })),
        })
    }

    // ── Music ───────────────────────────────────────────────────

    /// Start playing a track.  If the same track is already playing, do nothing.
    /// When muted, the track is loaded but paused — un-muting will resume it.
    pub fn play_track(&self, track: &Track) {
        let mut inner = self.inner.lock().unwrap();

        if inner.current_track == Some(track.name) && !inner.music_sink.empty() {
            return; // already playing (or paused with the same track)
        }

        inner.music_sink.stop();
        // Create a new sink for the new track
        if let Ok(sink) = Sink::try_new(&inner.stream_handle) {
            sink.set_volume(inner.music_volume);

            // Render several cycles and append them to loop
            let source = render_track(track, 16); // 16 cycles ≈ a nice loop
            sink.append(source);

            if inner.muted {
                sink.pause();
            }

            inner.music_sink = sink;
            inner.current_track = Some(track.name);
        }
    }

    /// Stop the current music track.
    pub fn stop_music(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.music_sink.stop();
        inner.current_track = None;
    }

    // ── SFX ─────────────────────────────────────────────────────

    /// Play a one-shot sound effect.
    pub fn play_sfx(&self, sfx: Sfx) {
        let mut inner = self.inner.lock().unwrap();

        if inner.muted {
            return;
        }

        // Garbage-collect finished sinks
        inner.sfx_sinks.retain(|s| !s.empty());

        let def = tracks::sfx_def(sfx);
        let source = render_sfx(&def);

        if let Ok(sink) = Sink::try_new(&inner.stream_handle) {
            sink.set_volume(inner.sfx_volume);
            sink.append(source);
            inner.sfx_sinks.push(sink);
        }
    }

    // ── Volume controls ─────────────────────────────────────────

    pub fn set_music_volume(&self, vol: f32) {
        let mut inner = self.inner.lock().unwrap();
        inner.music_volume = vol.clamp(0.0, 1.0);
        inner.music_sink.set_volume(inner.music_volume);
    }

    pub fn set_sfx_volume(&self, vol: f32) {
        let mut inner = self.inner.lock().unwrap();
        inner.sfx_volume = vol.clamp(0.0, 1.0);
    }

    pub fn music_volume(&self) -> f32 {
        self.inner.lock().unwrap().music_volume
    }

    pub fn sfx_volume(&self) -> f32 {
        self.inner.lock().unwrap().sfx_volume
    }

    pub fn toggle_mute(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.muted = !inner.muted;
        if inner.muted {
            inner.music_sink.pause();
        } else {
            inner.music_sink.play();
        }
    }

    pub fn set_muted(&self, muted: bool) {
        let mut inner = self.inner.lock().unwrap();
        inner.muted = muted;
        if muted {
            inner.music_sink.pause();
        } else {
            inner.music_sink.play();
        }
    }

    pub fn is_muted(&self) -> bool {
        self.inner.lock().unwrap().muted
    }
}

// ── Track rendering ─────────────────────────────────────────────

/// Render N cycles of a track into a single `Source`.
fn render_track(track: &Track, num_cycles: usize) -> Box<dyn Source<Item = f32> + Send> {
    let cycle_dur = track.cycle_duration();
    let sample_rate = 44100u32;
    let total_samples = (cycle_dur * num_cycles as f64 * sample_rate as f64) as usize;

    let mut buffer = vec![0.0f32; total_samples];

    for cycle in 0..num_cycles {
        let cycle_offset = (cycle as f64 * cycle_dur * sample_rate as f64) as usize;

        for voice in &track.voices {
            let events = voice.pattern.query(cycle);
            for event in &events {
                render_event_into(
                    &mut buffer,
                    event,
                    voice,
                    cycle_offset,
                    cycle_dur,
                    sample_rate,
                );
            }
        }
    }

    // Soft-clip to prevent distortion
    for s in &mut buffer {
        *s = s.clamp(-0.95, 0.95);
    }

    let source = rodio::buffer::SamplesBuffer::new(1, sample_rate, buffer);
    // Loop the rendered buffer forever for music
    Box::new(source.repeat_infinite())
}

/// Render a single SFX (one cycle).
fn render_sfx(def: &tracks::SfxDef) -> Box<dyn Source<Item = f32> + Send> {
    let cycle_dur = 4.0 * 60.0 / def.bpm;
    let sample_rate = 44100u32;
    let total_samples = (cycle_dur * sample_rate as f64) as usize;
    let mut buffer = vec![0.0f32; total_samples];

    let voice = Voice {
        pattern: def.pattern.clone(),
        waveform: def.waveform,
        amplitude: def.amplitude,
        is_drum: def.is_drum,
    };

    let events = def.pattern.query(0);
    for event in &events {
        render_event_into(&mut buffer, event, &voice, 0, cycle_dur, sample_rate);
    }

    for s in &mut buffer {
        *s = s.clamp(-0.95, 0.95);
    }

    Box::new(rodio::buffer::SamplesBuffer::new(1, sample_rate, buffer))
}

/// Mix a single event's audio into the output buffer.
fn render_event_into(
    buffer: &mut [f32],
    event: &Event,
    voice: &Voice,
    cycle_offset: usize,
    cycle_dur: f64,
    sample_rate: u32,
) {
    let start_sec = event.start * cycle_dur;
    let dur_sec = event.duration * cycle_dur;
    let note_dur = Duration::from_secs_f64(dur_sec.min(2.0)); // cap at 2s

    let start_sample = cycle_offset + (start_sec * sample_rate as f64) as usize;

    if voice.is_drum {
        // Choose drum type based on a pattern of kick/hihat/snare
        let drum_pos = (event.start * 4.0).round() as usize % 4;
        let source: Box<dyn Source<Item = f32> + Send> = match drum_pos {
            0 => Box::new(KickDrum::new(voice.amplitude)),
            2 => Box::new(Snare::new(voice.amplitude * 0.8)),
            _ => Box::new(Hihat::new(voice.amplitude * 0.6)),
        };
        mix_source_into(buffer, source, start_sample);
    } else {
        let osc = Oscillator::new(
            voice.waveform,
            event.note.freq() as f32,
            note_dur,
            voice.amplitude,
        );
        mix_source_into(buffer, Box::new(osc), start_sample);
    }
}

/// Additively mix a source into the output buffer.
fn mix_source_into(buffer: &mut [f32], source: Box<dyn Source<Item = f32> + Send>, start: usize) {
    for (i, sample) in source.into_iter().enumerate() {
        let idx = start + i;
        if idx < buffer.len() {
            buffer[idx] += sample;
        } else {
            break;
        }
    }
}
