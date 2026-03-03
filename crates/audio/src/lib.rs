//! `monster-battle-audio` — Strudel-inspired music & SFX engine.
//!
//! This crate provides a standalone audio system for the monster battle game.
//! Music is composed using a **mini-notation** parser inspired by
//! [Strudel](https://strudel.cc), then rendered in real-time through software
//! synthesis (oscillators + drum synths) via [`rodio`].
//!
//! # Architecture
//!
//! ```text
//!  ┌─────────────────────────────────────────────────┐
//!  │  pattern.rs  — mini-notation parser & evaluator  │
//!  └──────────┬──────────────────────────────────────┘
//!             │ Pattern → Vec<Event>
//!  ┌──────────▼──────────────────────────────────────┐
//!  │  synth.rs — Oscillator, KickDrum, Hihat, Snare  │
//!  └──────────┬──────────────────────────────────────┘
//!             │ rodio::Source
//!  ┌──────────▼──────────────────────────────────────┐
//!  │  tracks.rs — game tracks & SFX definitions       │
//!  └──────────┬──────────────────────────────────────┘
//!             │
//!  ┌──────────▼──────────────────────────────────────┐
//!  │  engine.rs — AudioEngine (rodio output + mixing) │
//!  └─────────────────────────────────────────────────┘
//! ```
//!
//! # Quick start
//!
//! ```rust,no_run
//! use monster_battle_audio::{AudioEngine, tracks};
//!
//! let engine = AudioEngine::try_new().expect("no audio device");
//! engine.play_track(&tracks::title_theme());
//! // …later…
//! engine.play_sfx(tracks::Sfx::Hit);
//! ```

pub mod engine;
pub mod pattern;
pub mod synth;
pub mod track_def;
pub mod tracks;

pub use engine::AudioEngine;
pub use track_def::TrackDef;
pub use tracks::Sfx;
