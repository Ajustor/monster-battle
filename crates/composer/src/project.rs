//! Serialisable project format for the composer.
//!
//! A project is simply an alias for [`TrackDef`] from the audio crate, plus
//! convenience helpers (default constructor, data directory, project listing).
//!
//! Since `Track.name` is now a `String`, the old `LEAKED_NAME` hack is gone.

use std::path::PathBuf;

use monster_battle_audio::track_def::{TrackDef, VoiceDef};

// ── Project = TrackDef ──────────────────────────────────────────

/// A composer project: identical to the built‑in track JSON format.
pub type Project = TrackDef;

// ── Convenience constructor ─────────────────────────────────────

/// Create a default empty project.
pub fn new_project() -> Project {
    Project {
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

// ── Data directory ──────────────────────────────────────────────

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
