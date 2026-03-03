//! Serialisable track definition format.
//!
//! A `TrackDef` describes a track as JSON (name, BPM, voices with
//! mini‑notation pattern strings).  It can be converted to a playable [`Track`]
//! via [`TrackDef::to_track`].
//!
//! This is the canonical exchange format shared between the Composer tool
//! and the game's built‑in tracks (which are embedded as JSON files under
//! `crates/audio/tracks/`).

use serde::{Deserialize, Serialize};

use crate::pattern;
use crate::synth::Waveform;
use crate::tracks::{Track, Voice};

// ── Serialisable types ──────────────────────────────────────────

/// A single voice definition (JSON‑friendly).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceDef {
    pub pattern: String,
    pub waveform: String,
    pub amplitude: f32,
    #[serde(default)]
    pub is_drum: bool,
}

/// A complete track definition (JSON‑friendly).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackDef {
    pub name: String,
    pub bpm: f64,
    pub voices: Vec<VoiceDef>,
}

// ── Waveform helpers ────────────────────────────────────────────

pub fn waveform_to_str(wf: Waveform) -> &'static str {
    match wf {
        Waveform::Sine => "sine",
        Waveform::Square => "square",
        Waveform::Sawtooth => "sawtooth",
        Waveform::Triangle => "triangle",
    }
}

pub fn str_to_waveform(s: &str) -> Waveform {
    match s.to_lowercase().as_str() {
        "square" => Waveform::Square,
        "sawtooth" | "saw" => Waveform::Sawtooth,
        "triangle" | "tri" => Waveform::Triangle,
        _ => Waveform::Sine,
    }
}

/// All available waveform names paired with their enum values.
pub const WAVEFORMS: [(&str, Waveform); 4] = [
    ("sine", Waveform::Sine),
    ("square", Waveform::Square),
    ("sawtooth", Waveform::Sawtooth),
    ("triangle", Waveform::Triangle),
];

// ── Conversion ──────────────────────────────────────────────────

impl TrackDef {
    /// Convert to a playable [`Track`].
    pub fn to_track(&self) -> Track {
        Track {
            name: self.name.clone(),
            bpm: self.bpm,
            voices: self
                .voices
                .iter()
                .map(|v| Voice {
                    pattern: pattern::parse(&v.pattern),
                    waveform: str_to_waveform(&v.waveform),
                    amplitude: v.amplitude,
                    is_drum: v.is_drum,
                })
                .collect(),
        }
    }

    /// Parse from a JSON string.
    pub fn from_json(json: &str) -> Self {
        serde_json::from_str(json).expect("invalid track JSON")
    }

    /// Serialise to a pretty‑printed JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("serialisation failed")
    }

    /// Save to a JSON file.
    pub fn save(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load from a JSON file.
    pub fn load(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let def: Self = serde_json::from_str(&json)?;
        Ok(def)
    }

    /// Build a `TrackDef` from a playable `Track`.
    ///
    /// **Note:** the original pattern mini‑notation is lost; a placeholder
    /// pattern is used.  This is mainly useful for extracting metadata.
    pub fn from_track(track: &Track) -> Self {
        Self {
            name: track.name.clone(),
            bpm: track.bpm,
            voices: track
                .voices
                .iter()
                .map(|v| VoiceDef {
                    pattern: String::from("c4 e4 g4 c5"),
                    waveform: waveform_to_str(v.waveform).to_string(),
                    amplitude: v.amplitude,
                    is_drum: v.is_drum,
                })
                .collect(),
        }
    }
}
