//! Serialisable project format for the composer.
//!
//! A project is a JSON file describing a track (name, BPM, voices) using
//! the same mini‑notation understood by `monster_battle_audio::pattern::parse`.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use monster_battle_audio::pattern;
use monster_battle_audio::synth::Waveform;
use monster_battle_audio::tracks::{Track, Voice};

// ── Serialisable mirror types ───────────────────────────────────

/// A single voice as stored on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceDef {
    /// Mini‑notation pattern string.
    pub pattern: String,
    /// Waveform name (sine, square, sawtooth, triangle).
    pub waveform: String,
    /// Amplitude 0.0–1.0.
    pub amplitude: f32,
    /// Whether this voice is a drum track.
    pub is_drum: bool,
}

/// A complete project (track) as stored on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub bpm: f64,
    pub voices: Vec<VoiceDef>,
}

// ── Conversions ─────────────────────────────────────────────────

#[allow(dead_code)]
fn waveform_to_str(w: Waveform) -> &'static str {
    match w {
        Waveform::Sine => "sine",
        Waveform::Square => "square",
        Waveform::Sawtooth => "sawtooth",
        Waveform::Triangle => "triangle",
    }
}

fn str_to_waveform(s: &str) -> Waveform {
    match s.to_lowercase().as_str() {
        "square" => Waveform::Square,
        "sawtooth" | "saw" => Waveform::Sawtooth,
        "triangle" | "tri" => Waveform::Triangle,
        _ => Waveform::Sine,
    }
}

pub const WAVEFORMS: [(&str, Waveform); 4] = [
    ("sine", Waveform::Sine),
    ("square", Waveform::Square),
    ("sawtooth", Waveform::Sawtooth),
    ("triangle", Waveform::Triangle),
];

impl Project {
    /// Create a default empty project.
    pub fn new() -> Self {
        Self {
            name: String::from("untitled"),
            bpm: 120.0,
            voices: vec![VoiceDef {
                pattern: String::from("c4 e4 g4 c5"),
                waveform: String::from("triangle"),
                amplitude: 0.20,
                is_drum: false,
            }],
        }
    }

    /// Convert to a playable `Track`.
    ///
    /// The track name is leaked to a `&'static str` so the audio engine can
    /// hold it.  This is fine for a composer tool with a bounded number of
    /// previews.
    pub fn to_track(&self) -> Track {
        let name: &'static str = Box::leak(self.name.clone().into_boxed_str());
        Track {
            name,
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

    /// Load a project from a built‑in track definition.
    #[allow(dead_code)]
    pub fn from_builtin(track: &Track) -> Self {
        // We cannot reverse‑engineer the pattern string from the AST, so
        // built‑in presets are hard‑coded below via `builtin_presets()`.
        Self {
            name: track.name.to_string(),
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

    // ── Persistence ─────────────────────────────────────────────

    /// Save the project to a JSON file.
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load a project from a JSON file.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let project: Self = serde_json::from_str(&json)?;
        Ok(project)
    }
}

/// Default data directory for composer projects.
pub fn projects_dir() -> PathBuf {
    let base = if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(xdg)
    } else if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".local").join("share")
    } else {
        PathBuf::from(".")
    };
    base.join("monster-battle").join("composer")
}

/// List saved project files in the data directory.
pub fn list_projects() -> Vec<PathBuf> {
    let dir = projects_dir();
    if !dir.exists() {
        return Vec::new();
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    files
}
