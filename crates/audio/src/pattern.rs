//! Strudel-inspired mini-notation parser and pattern evaluation engine.
//!
//! Supports a subset of Strudel's mini-notation:
//! - **Sequences**: `"c4 e4 g4"` — space-separated events, equally spaced in one cycle
//! - **Groups**: `"[c4 e4]"` — brackets group elements into a single time slot
//! - **Alternation**: `"<c4 e4 g4>"` — one element per cycle, cycling through
//! - **Rests**: `"~"` — silence
//! - **Speed**: `"[c4 e4]*2"` — play pattern N times per cycle
//! - **Polyphony**: `"c4 e4, g4 b4"` — comma-separated layers play simultaneously

use std::fmt;

// ── Note ────────────────────────────────────────────────────────

/// A musical note represented by its MIDI number (0–127).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Note {
    pub midi: u8,
}

/// Sentinel value used for percussion hits (the pattern triggers a drum sound).
pub const DRUM_HIT: Note = Note { midi: 0 };

impl Note {
    pub fn new(midi: u8) -> Self {
        Self { midi }
    }

    /// Convert MIDI note to frequency in Hz.
    /// A4 (MIDI 69) = 440 Hz.
    pub fn freq(&self) -> f64 {
        440.0 * 2.0_f64.powf((self.midi as f64 - 69.0) / 12.0)
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = [
            "C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
        ];
        let octave = (self.midi / 12) as i8 - 1;
        let name = names[(self.midi % 12) as usize];
        write!(f, "{}{}", name, octave)
    }
}

// ── Pattern AST ─────────────────────────────────────────────────

/// A rhythmic / melodic pattern, inspired by Strudel's mini-notation.
#[derive(Debug, Clone)]
pub enum Pattern {
    /// A single note event.
    Note(Note),
    /// Silence — produces no event.
    Rest,
    /// A sequence of patterns, equally distributed within the time slot.
    Sequence(Vec<Pattern>),
    /// One child per cycle, cycling through the list.
    Alternate(Vec<Pattern>),
    /// All children play simultaneously (polyphony).
    Stack(Vec<Pattern>),
    /// Play the inner pattern sped up by the given factor.
    Speed(Box<Pattern>, f64),
}

// ── Event ───────────────────────────────────────────────────────

/// A concrete note event produced by querying a pattern at a given cycle.
#[derive(Debug, Clone)]
pub struct Event {
    /// Start position within the cycle, in \[0.0, 1.0).
    pub start: f64,
    /// Duration as a fraction of the cycle.
    pub duration: f64,
    /// The note to play.
    pub note: Note,
}

// ── Pattern evaluation ──────────────────────────────────────────

impl Pattern {
    /// Query the pattern for all events at the given cycle number.
    ///
    /// Returns events with positions normalised to \[0.0, 1.0) within the cycle.
    pub fn query(&self, cycle: usize) -> Vec<Event> {
        match self {
            Pattern::Note(note) => {
                vec![Event {
                    start: 0.0,
                    duration: 1.0,
                    note: *note,
                }]
            }

            Pattern::Rest => vec![],

            Pattern::Sequence(children) => {
                if children.is_empty() {
                    return vec![];
                }
                let n = children.len() as f64;
                let step = 1.0 / n;
                children
                    .iter()
                    .enumerate()
                    .flat_map(|(i, child)| {
                        child.query(cycle).into_iter().map(move |mut e| {
                            e.start = e.start * step + i as f64 * step;
                            e.duration *= step;
                            e
                        })
                    })
                    .collect()
            }

            Pattern::Alternate(children) => {
                if children.is_empty() {
                    return vec![];
                }
                let idx = cycle % children.len();
                children[idx].query(cycle)
            }

            Pattern::Stack(children) => children
                .iter()
                .flat_map(|child| child.query(cycle))
                .collect(),

            Pattern::Speed(child, factor) => {
                if *factor <= 0.0 {
                    return vec![];
                }
                let n = factor.ceil() as usize;
                (0..n)
                    .flat_map(|i| {
                        let sub_cycle = cycle * n + i;
                        let factor = *factor;
                        child.query(sub_cycle).into_iter().filter_map(move |mut e| {
                            e.start = e.start / factor + i as f64 / factor;
                            e.duration /= factor;
                            if e.start < 1.0 { Some(e) } else { None }
                        })
                    })
                    .collect()
            }
        }
    }
}

// ── Mini-notation parser ────────────────────────────────────────

/// Parse a Strudel-like mini-notation string into a [`Pattern`].
///
/// # Examples
/// ```
/// use monster_battle_audio::pattern::parse;
///
/// let pat = parse("c4 e4 g4");       // sequence of 3 notes
/// let pat = parse("<c4 e4 g4>");      // one note per cycle
/// let pat = parse("[c4 e4]*2");       // group played twice per cycle
/// let pat = parse("c4 e4, g4 b4");   // two layers in parallel
/// let pat = parse("x ~ x ~");        // drum rhythm
/// ```
pub fn parse(input: &str) -> Pattern {
    let mut parser = Parser::new(input);
    parser.parse_top()
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    // ── Grammar ─────────────────────────────────────────────────

    fn parse_top(&mut self) -> Pattern {
        self.parse_stack()
    }

    /// Comma-separated parallel layers.
    fn parse_stack(&mut self) -> Pattern {
        let first = self.parse_sequence();
        let mut layers = vec![first];

        while self.peek() == Some(',') {
            self.advance(); // consume ','
            layers.push(self.parse_sequence());
        }

        if layers.len() == 1 {
            layers.pop().unwrap()
        } else {
            Pattern::Stack(layers)
        }
    }

    /// Space-separated atoms forming a sequence.
    fn parse_sequence(&mut self) -> Pattern {
        self.skip_whitespace();
        let mut items = Vec::new();

        loop {
            self.skip_whitespace();
            match self.peek() {
                None | Some(',') | Some(']') | Some('>') => break,
                _ => items.push(self.parse_atom()),
            }
        }

        match items.len() {
            0 => Pattern::Rest,
            1 => items.pop().unwrap(),
            _ => Pattern::Sequence(items),
        }
    }

    /// A single element with optional modifiers (`*N`).
    fn parse_atom(&mut self) -> Pattern {
        self.skip_whitespace();

        let mut pattern = match self.peek() {
            Some('[') => self.parse_group(),
            Some('<') => self.parse_alternate(),
            Some('~') => {
                self.advance();
                Pattern::Rest
            }
            Some(_) => self.parse_note_or_hit(),
            None => Pattern::Rest,
        };

        // Parse modifiers: *N (speed)
        loop {
            match self.peek() {
                Some('*') => {
                    self.advance();
                    let factor = self.parse_number();
                    pattern = Pattern::Speed(Box::new(pattern), factor);
                }
                _ => break,
            }
        }

        pattern
    }

    /// `[...]` — grouped sub-pattern (occupies one time slot in parent).
    fn parse_group(&mut self) -> Pattern {
        self.advance(); // consume '['
        let inner = self.parse_stack();
        self.skip_whitespace();
        if self.peek() == Some(']') {
            self.advance();
        }
        inner
    }

    /// `<...>` — alternation (one child per cycle).
    fn parse_alternate(&mut self) -> Pattern {
        self.advance(); // consume '<'
        let mut items = Vec::new();

        loop {
            self.skip_whitespace();
            match self.peek() {
                None | Some('>') => break,
                _ => items.push(self.parse_atom()),
            }
        }

        if self.peek() == Some('>') {
            self.advance();
        }

        if items.is_empty() {
            Pattern::Rest
        } else {
            Pattern::Alternate(items)
        }
    }

    /// Parse a note (`c4`, `eb3`, `f#5`) or a drum hit (`x`).
    fn parse_note_or_hit(&mut self) -> Pattern {
        // Drum hit: 'x' followed by non-alpha (or end)
        if self.peek() == Some('x') && self.peek_at(1).map_or(true, |c| !c.is_alphabetic()) {
            self.advance();
            return Pattern::Note(DRUM_HIT);
        }

        // Read base note letter (a–g)
        let note_char = match self.peek() {
            Some(c) if ('a'..='g').contains(&c.to_ascii_lowercase()) => {
                self.advance();
                c.to_ascii_lowercase()
            }
            _ => {
                // Unknown token — skip and return rest
                self.advance();
                return Pattern::Rest;
            }
        };

        let base: i16 = match note_char {
            'c' => 0,
            'd' => 2,
            'e' => 4,
            'f' => 5,
            'g' => 7,
            'a' => 9,
            'b' => 11,
            _ => unreachable!(),
        };

        // Accidental: '#' → sharp, 'b' → flat
        let mut accidental: i16 = 0;
        match self.peek() {
            Some('#') => {
                self.advance();
                accidental = 1;
            }
            Some('b') => {
                if note_char == 'b' {
                    // "bb" — flat only if followed by a digit (otherwise it's a new note)
                    if self.peek_at(1).map_or(false, |c| c.is_ascii_digit()) {
                        self.advance();
                        accidental = -1;
                    }
                } else {
                    // After c/d/e/f/g/a, 'b' is always a flat modifier
                    self.advance();
                    accidental = -1;
                }
            }
            _ => {}
        }

        // Octave number (default 4 if absent)
        let octave: i16 = if self.peek().map_or(false, |c| c.is_ascii_digit()) {
            let mut num_str = String::new();
            while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                num_str.push(self.advance().unwrap());
            }
            num_str.parse().unwrap_or(4)
        } else {
            4
        };

        let midi = (octave + 1) * 12 + base + accidental;
        if (0..=127).contains(&midi) {
            Pattern::Note(Note { midi: midi as u8 })
        } else {
            Pattern::Rest
        }
    }

    /// Parse a decimal number (e.g. `2`, `2.5`).
    fn parse_number(&mut self) -> f64 {
        let mut s = String::new();
        while self
            .peek()
            .map_or(false, |c| c.is_ascii_digit() || c == '.')
        {
            s.push(self.advance().unwrap());
        }
        s.parse().unwrap_or(1.0)
    }
}

// ── Tests ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_note() {
        let events = parse("c4").query(0);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].note.midi, 60); // C4
    }

    #[test]
    fn sequence_of_three() {
        let events = parse("c4 e4 g4").query(0);
        assert_eq!(events.len(), 3);
        assert!((events[0].start).abs() < 1e-9);
        assert!((events[1].start - 1.0 / 3.0).abs() < 1e-9);
        assert!((events[2].start - 2.0 / 3.0).abs() < 1e-9);
        assert!((events[0].duration - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn rest_produces_no_event() {
        let events = parse("c4 ~ g4").query(0);
        assert_eq!(events.len(), 2); // only c4 and g4
    }

    #[test]
    fn alternation_cycles() {
        let pat = parse("<c4 e4 g4>");
        assert_eq!(pat.query(0)[0].note.midi, 60); // C4
        assert_eq!(pat.query(1)[0].note.midi, 64); // E4
        assert_eq!(pat.query(2)[0].note.midi, 67); // G4
        assert_eq!(pat.query(3)[0].note.midi, 60); // wraps
    }

    #[test]
    fn group_subdivides() {
        let events = parse("c4 [e4 g4]").query(0);
        assert_eq!(events.len(), 3);
        // c4: 0.0..0.5
        assert!((events[0].start).abs() < 1e-9);
        assert!((events[0].duration - 0.5).abs() < 1e-9);
        // e4: 0.5..0.75
        assert!((events[1].start - 0.5).abs() < 1e-9);
        assert!((events[1].duration - 0.25).abs() < 1e-9);
        // g4: 0.75..1.0
        assert!((events[2].start - 0.75).abs() < 1e-9);
    }

    #[test]
    fn speed_doubles() {
        let events = parse("[c4 e4]*2").query(0);
        assert_eq!(events.len(), 4); // 2 notes × 2 repetitions
    }

    #[test]
    fn polyphony_stacks() {
        let events = parse("c4, e4").query(0);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].note.midi, 60);
        assert_eq!(events[1].note.midi, 64);
    }

    #[test]
    fn sharps_and_flats() {
        assert_eq!(parse("c#4").query(0)[0].note.midi, 61);
        assert_eq!(parse("eb4").query(0)[0].note.midi, 63);
        assert_eq!(parse("bb3").query(0)[0].note.midi, 58); // Bb3
        assert_eq!(parse("f#5").query(0)[0].note.midi, 78);
    }

    #[test]
    fn note_frequencies() {
        let a4 = Note::new(69);
        assert!((a4.freq() - 440.0).abs() < 0.01);
        let c4 = Note::new(60);
        assert!((c4.freq() - 261.63).abs() < 0.1);
    }

    #[test]
    fn drum_hit() {
        let events = parse("x ~ x ~").query(0);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].note, DRUM_HIT);
    }

    #[test]
    fn nested_alternate_in_group() {
        let pat = parse("<[a3 c4 e4 c4] [f3 a3 c4 a3]>");
        let c0 = pat.query(0);
        assert_eq!(c0.len(), 4); // first group
        assert_eq!(c0[0].note.midi, 57); // A3

        let c1 = pat.query(1);
        assert_eq!(c1.len(), 4); // second group
        assert_eq!(c1[0].note.midi, 53); // F3
    }
}
