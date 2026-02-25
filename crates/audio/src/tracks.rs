//! Game music tracks and sound effects, defined with Strudel mini-notation.
//!
//! Each track is a set of **voices** (melody, bass, chords, drums) that play in
//! parallel.  The engine renders them cycle by cycle.

use crate::pattern::{self, Pattern};
use crate::synth::Waveform;

// ── Voice ───────────────────────────────────────────────────────

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
    pub name: &'static str,
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

// ── Built-in tracks ─────────────────────────────────────────────

/// Title screen / main menu — adventurous chiptune with a heroic feel.
pub fn title_theme() -> Track {
    Track {
        name: "title",
        bpm: 138.0,
        voices: vec![
            // Hero melody — triangle, bold rising phrase with swing
            Voice {
                pattern: pattern::parse(
                    "<\
                    [g4 b4 d5 [g5 f#5]] \
                    [e5 [d5 c5] b4 [a4 b4]] \
                    [c5 e5 [g5 a5] g5] \
                    [f#5 d5 b4 ~]\
                    >",
                ),
                waveform: Waveform::Triangle,
                amplitude: 0.20,
                is_drum: false,
            },
            // Counter-melody — sine, echoing response an octave lower
            Voice {
                pattern: pattern::parse(
                    "<\
                    [~ ~ g4 [b4 a4]] \
                    [~ ~ e4 [d4 c4]] \
                    [~ ~ c4 [e4 g4]] \
                    [~ ~ d4 [f#4 ~]]\
                    >",
                ),
                waveform: Waveform::Sine,
                amplitude: 0.10,
                is_drum: false,
            },
            // Bass — sawtooth, driving root-fifth movement
            Voice {
                pattern: pattern::parse(
                    "<\
                    [g2 ~ d3 g2] \
                    [c3 ~ g2 c3] \
                    [a2 ~ e3 a2] \
                    [d3 ~ a2 d3]\
                    >",
                ),
                waveform: Waveform::Sawtooth,
                amplitude: 0.14,
                is_drum: false,
            },
            // Power chords — square, staccato stabs on the beat
            Voice {
                pattern: pattern::parse(
                    "<\
                    [g3,b3,d4] ~ [g3,b3,d4] ~ \
                    [c4,e4,g4] ~ [c4,e4,g4] ~ \
                    [a3,c4,e4] ~ [a3,c4,e4] ~ \
                    [d4,f#4,a4] ~ [d4,f#4,a4] ~\
                    >",
                ),
                waveform: Waveform::Square,
                amplitude: 0.06,
                is_drum: false,
            },
            // Drums — punchy march beat
            Voice {
                pattern: pattern::parse("[x ~ [~ x] ~] [x ~ x [x x]]"),
                waveform: Waveform::Sine,
                amplitude: 0.14,
                is_drum: true,
            },
        ],
    }
}

/// Battle theme — fast, intense, darker.
pub fn battle_theme() -> Track {
    Track {
        name: "battle",
        bpm: 155.0,
        voices: vec![
            // Aggressive lead — sawtooth
            Voice {
                pattern: pattern::parse(
                    "<\
                    [a4 a4 [c5 d5] a4] \
                    [g4 g4 [bb4 c5] g4] \
                    [f4 f4 [a4 bb4] f4] \
                    [g4 ~ [bb4 c5] d5]\
                    >",
                ),
                waveform: Waveform::Sawtooth,
                amplitude: 0.18,
                is_drum: false,
            },
            // Bass — pulsing
            Voice {
                pattern: pattern::parse(
                    "<\
                    [a2 ~ a2 ~] \
                    [g2 ~ g2 ~] \
                    [f2 ~ f2 ~] \
                    [g2 ~ g2 a2]\
                    >",
                ),
                waveform: Waveform::Square,
                amplitude: 0.16,
                is_drum: false,
            },
            // Power chords — square stabs
            Voice {
                pattern: pattern::parse(
                    "<\
                    [a3,e4] ~ [a3,e4] ~ \
                    [g3,d4] ~ [g3,d4] ~ \
                    [f3,c4] ~ [f3,c4] ~ \
                    [g3,d4] ~ [g3,d4] ~\
                    >",
                ),
                waveform: Waveform::Square,
                amplitude: 0.10,
                is_drum: false,
            },
            // Fast drums
            Voice {
                pattern: pattern::parse("[x ~ x ~]*2"),
                waveform: Waveform::Sine,
                amplitude: 0.18,
                is_drum: true,
            },
        ],
    }
}

/// Victory fanfare — triumphant, short jingle.
pub fn victory_fanfare() -> Track {
    Track {
        name: "victory",
        bpm: 140.0,
        voices: vec![
            Voice {
                pattern: pattern::parse(
                    "<\
                    [c5 e5 g5 [c6 c6]] \
                    [c6 ~ ~ ~]\
                    >",
                ),
                waveform: Waveform::Triangle,
                amplitude: 0.24,
                is_drum: false,
            },
            Voice {
                pattern: pattern::parse(
                    "<\
                    [c3 e3 g3 c4] \
                    [c4 ~ ~ ~]\
                    >",
                ),
                waveform: Waveform::Square,
                amplitude: 0.14,
                is_drum: false,
            },
            Voice {
                pattern: pattern::parse("x ~ x [x x]"),
                waveform: Waveform::Sine,
                amplitude: 0.14,
                is_drum: true,
            },
        ],
    }
}

/// Defeat — somber, minor.
pub fn defeat_theme() -> Track {
    Track {
        name: "defeat",
        bpm: 80.0,
        voices: vec![
            Voice {
                pattern: pattern::parse(
                    "<\
                    [e4 ~ d4 c4] \
                    [b3 ~ a3 ~]\
                    >",
                ),
                waveform: Waveform::Sine,
                amplitude: 0.18,
                is_drum: false,
            },
            Voice {
                pattern: pattern::parse(
                    "<\
                    [a2 ~ ~ ~] \
                    [e2 ~ ~ ~]\
                    >",
                ),
                waveform: Waveform::Triangle,
                amplitude: 0.12,
                is_drum: false,
            },
        ],
    }
}

/// Monster list / peaceful exploration.
pub fn exploration_theme() -> Track {
    Track {
        name: "exploration",
        bpm: 100.0,
        voices: vec![
            // Gentle melody
            Voice {
                pattern: pattern::parse(
                    "<\
                    [c5 e5 g5 e5] \
                    [d5 f5 a5 f5] \
                    [e5 g5 b5 g5] \
                    [f5 a5 c6 a5]\
                    >",
                ),
                waveform: Waveform::Triangle,
                amplitude: 0.16,
                is_drum: false,
            },
            // Soft arpeggiated chords
            Voice {
                pattern: pattern::parse(
                    "<\
                    [c3 e3 g3 e3] \
                    [d3 f3 a3 f3] \
                    [e3 g3 b3 g3] \
                    [f3 a3 c4 a3]\
                    >",
                ),
                waveform: Waveform::Sine,
                amplitude: 0.10,
                is_drum: false,
            },
        ],
    }
}

/// Breeding / genetics — mystical, intertwining DNA helices.
pub fn breeding_theme() -> Track {
    Track {
        name: "breeding",
        bpm: 76.0,
        voices: vec![
            // DNA Helix α — ascending melodic strand (sine, warm)
            Voice {
                pattern: pattern::parse(
                    "<\
                    [d5 ~ f#5 ~] \
                    [g5 ~ a5 ~] \
                    [b5 ~ a5 ~] \
                    [g5 ~ f#5 d5]\
                    >",
                ),
                waveform: Waveform::Sine,
                amplitude: 0.16,
                is_drum: false,
            },
            // DNA Helix β — counter-melody, descending (triangle)
            Voice {
                pattern: pattern::parse(
                    "<\
                    [~ a4 ~ d4] \
                    [~ b4 ~ e4] \
                    [~ d5 ~ b4] \
                    [~ a4 ~ g4]\
                    >",
                ),
                waveform: Waveform::Triangle,
                amplitude: 0.11,
                is_drum: false,
            },
            // Magic arpeggio — shimmering cascading notes (sine)
            Voice {
                pattern: pattern::parse(
                    "<\
                    [d3 f#3 a3 d4] \
                    [g3 b3 d4 g4] \
                    [a3 c#4 e4 a4] \
                    [g3 b3 d4 g4]\
                    >",
                ),
                waveform: Waveform::Sine,
                amplitude: 0.07,
                is_drum: false,
            },
            // Heartbeat bass — the pulse of creation (triangle, gentle)
            Voice {
                pattern: pattern::parse(
                    "<\
                    [d2 ~ d2 ~] \
                    [g2 ~ g2 ~] \
                    [a2 ~ a2 ~] \
                    [g2 ~ g2 ~]\
                    >",
                ),
                waveform: Waveform::Triangle,
                amplitude: 0.12,
                is_drum: false,
            },
            // Sparkle pad — ethereal lydian chord shimmer (square, almost subliminal)
            Voice {
                pattern: pattern::parse(
                    "<\
                    [d4,f#4,a4,c#5] \
                    [g4,b4,d5] \
                    [a4,c#5,e5] \
                    [g4,b4,d5,f#5]\
                    >",
                ),
                waveform: Waveform::Square,
                amplitude: 0.03,
                is_drum: false,
            },
            // Pulse — subtle heartbeat drum
            Voice {
                pattern: pattern::parse("[x ~ ~ ~] [~ ~ x ~]"),
                waveform: Waveform::Sine,
                amplitude: 0.06,
                is_drum: true,
            },
        ],
    }
}

/// Cemetery — sad, eerie Lavender Town vibe.
pub fn cemetery_theme() -> Track {
    Track {
        name: "cemetery",
        bpm: 68.0,
        voices: vec![
            // Haunting lead — high sine, sparse and melancholic
            Voice {
                pattern: pattern::parse(
                    "<\
                    [b4 ~ a4 ~] \
                    [g4 ~ f#4 ~] \
                    [e4 ~ d4 ~] \
                    [c4 ~ b3 ~]\
                    >",
                ),
                waveform: Waveform::Sine,
                amplitude: 0.14,
                is_drum: false,
            },
            // Ghost melody — detuned triangle an octave higher, very quiet
            Voice {
                pattern: pattern::parse(
                    "<\
                    [~ e5 ~ d5] \
                    [~ c5 ~ b4] \
                    [~ a4 ~ g4] \
                    [~ f#4 ~ e4]\
                    >",
                ),
                waveform: Waveform::Triangle,
                amplitude: 0.07,
                is_drum: false,
            },
            // Deep bass — ominous sustained notes
            Voice {
                pattern: pattern::parse(
                    "<\
                    [e2 ~ ~ ~] \
                    [c2 ~ ~ ~] \
                    [a1 ~ ~ ~] \
                    [b1 ~ ~ ~]\
                    >",
                ),
                waveform: Waveform::Triangle,
                amplitude: 0.10,
                is_drum: false,
            },
            // Eerie pad — minor chord, very soft square
            Voice {
                pattern: pattern::parse(
                    "<\
                    [e3,g3,b3] \
                    [c3,e3,g3] \
                    [a2,c3,e3] \
                    [b2,d3,f#3]\
                    >",
                ),
                waveform: Waveform::Square,
                amplitude: 0.04,
                is_drum: false,
            },
        ],
    }
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
