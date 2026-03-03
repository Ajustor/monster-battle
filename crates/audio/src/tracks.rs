//! Game music tracks and sound effects.
//!
//! Built‑in tracks are stored as JSON files in the `tracks/` directory and
//! embedded at compile time via `include_str!`.  Adding a new track is as
//! simple as dropping a `.json` file there and adding a one‑liner function
//! below.
//!
//! The JSON format matches [`TrackDef`](crate::track_def::TrackDef) —
//! the same format the Composer tool saves and loads.

use crate::pattern::{self, Pattern};
use crate::synth::Waveform;
use crate::track_def::TrackDef;

// ── Voice / Track structs ───────────────────────────────────────

/// A single musical voice: a pattern + waveform + volume.
#[derive(Clone)]
pub struct Voice {
    pub pattern: Pattern,
    pub waveform: Waveform,
    pub amplitude: f32,
    /// If true, this voice uses drum synthesis instead of oscillators.
    pub is_drum: bool,
}

/// A complete music track: BPM + a set of voices.
#[derive(Clone)]
pub struct Track {
    pub name: String,
    pub bpm: f64,
    pub voices: Vec<Voice>,
}

impl Track {
    /// Duration of a single cycle in seconds (= 1 bar in 4/4 at the given BPM).
    pub fn cycle_duration(&self) -> f64 {
        // One cycle = 1 bar = 4 beats at the given BPM.
        4.0 * 60.0 / self.bpm
    }
}

// ── Built‑in tracks (loaded from embedded JSON) ─────────────────

/// Title screen / main menu — adventurous chiptune with a heroic feel.
pub fn title_theme() -> Track {
    TrackDef::from_json(include_str!("../tracks/title.json")).to_track()
}

/// Battle theme — fast, intense, darker.
pub fn battle_theme() -> Track {
    TrackDef::from_json(include_str!("../tracks/battle.json")).to_track()
}

/// Victory fanfare — triumphant, short jingle.
pub fn victory_fanfare() -> Track {
    TrackDef::from_json(include_str!("../tracks/victory.json")).to_track()
}

/// Defeat — somber, minor.
pub fn defeat_theme() -> Track {
    TrackDef::from_json(include_str!("../tracks/defeat.json")).to_track()
}

/// Monster list / peaceful exploration.
pub fn exploration_theme() -> Track {
    TrackDef::from_json(include_str!("../tracks/exploration.json")).to_track()
}

/// Breeding / genetics — mystical, intertwining DNA helices.
pub fn breeding_theme() -> Track {
    TrackDef::from_json(include_str!("../tracks/breeding.json")).to_track()
}

/// Cemetery — sad, eerie Lavender Town vibe.
pub fn cemetery_theme() -> Track {
    TrackDef::from_json(include_str!("../tracks/cemetery.json")).to_track()
}

// ── Sound effects ───────────────────────────────────────────────

/// Identifiers for one-shot sound effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sfx {
    /// Attack / hit landing.
    Hit,
    /// Critical hit — more punch.
    CriticalHit,
    /// Menu navigation click.
    MenuSelect,
    /// Menu cursor move.
    MenuMove,
    /// Level up jingle.
    LevelUp,
    /// Monster dies.
    MonsterDeath,
    /// Heal / potion.
    Heal,
    /// Queue matched — PvP opponent found.
    MatchFound,
    /// Flee from combat.
    Flee,
}

/// SFX definition: a short pattern + waveform + amplitude.
pub struct SfxDef {
    pub pattern: Pattern,
    pub waveform: Waveform,
    pub amplitude: f32,
    pub bpm: f64,
    pub is_drum: bool,
}

/// Get the SFX definition for a given effect.
pub fn sfx_def(sfx: Sfx) -> SfxDef {
    match sfx {
        Sfx::Hit => SfxDef {
            pattern: pattern::parse("[c3 ~ a2 ~]"),
            waveform: Waveform::Square,
            amplitude: 0.25,
            bpm: 600.0,
            is_drum: false,
        },
        Sfx::CriticalHit => SfxDef {
            pattern: pattern::parse("[c3 e3 g3 c4]"),
            waveform: Waveform::Sawtooth,
            amplitude: 0.28,
            bpm: 800.0,
            is_drum: false,
        },
        Sfx::MenuSelect => SfxDef {
            pattern: pattern::parse("[e5 g5]"),
            waveform: Waveform::Triangle,
            amplitude: 0.15,
            bpm: 900.0,
            is_drum: false,
        },
        Sfx::MenuMove => SfxDef {
            pattern: pattern::parse("e5"),
            waveform: Waveform::Triangle,
            amplitude: 0.10,
            bpm: 1200.0,
            is_drum: false,
        },
        Sfx::LevelUp => SfxDef {
            pattern: pattern::parse("[c5 e5 g5 c6]"),
            waveform: Waveform::Triangle,
            amplitude: 0.22,
            bpm: 400.0,
            is_drum: false,
        },
        Sfx::MonsterDeath => SfxDef {
            pattern: pattern::parse("[e4 d4 c4 b3 a3]"),
            waveform: Waveform::Sawtooth,
            amplitude: 0.20,
            bpm: 300.0,
            is_drum: false,
        },
        Sfx::Heal => SfxDef {
            pattern: pattern::parse("[c5 d5 e5 f5 g5]"),
            waveform: Waveform::Sine,
            amplitude: 0.18,
            bpm: 500.0,
            is_drum: false,
        },
        Sfx::MatchFound => SfxDef {
            pattern: pattern::parse("[g4 c5 e5 g5]"),
            waveform: Waveform::Square,
            amplitude: 0.20,
            bpm: 500.0,
            is_drum: false,
        },
        Sfx::Flee => SfxDef {
            pattern: pattern::parse("[g4 f4 e4 d4 c4]"),
            waveform: Waveform::Triangle,
            amplitude: 0.16,
            bpm: 600.0,
            is_drum: false,
        },
    }
}
