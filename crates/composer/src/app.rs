//! Application state and main event loop for the composer TUI.

use std::io;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use monster_battle_audio::AudioEngine;

use crate::project::{self, Project, VoiceDef, WAVEFORMS};
use crate::ui;

// ── Focus / modes ───────────────────────────────────────────────

/// Which panel currently has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    /// The voice list on the left.
    VoiceList,
    /// The detail/editor panel on the right.
    Editor,
    /// The track‑level controls (name, BPM) at the top.
    TrackHeader,
}

/// Which field is being edited inside the editor panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorField {
    Pattern,
    Waveform,
    Amplitude,
    IsDrum,
}

impl EditorField {
    pub fn next(self) -> Self {
        match self {
            Self::Pattern => Self::Waveform,
            Self::Waveform => Self::Amplitude,
            Self::Amplitude => Self::IsDrum,
            Self::IsDrum => Self::Pattern,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Pattern => Self::IsDrum,
            Self::Waveform => Self::Pattern,
            Self::Amplitude => Self::Waveform,
            Self::IsDrum => Self::Amplitude,
        }
    }
}

/// Which header field is being edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderField {
    Name,
    Bpm,
}

impl HeaderField {
    pub fn next(self) -> Self {
        match self {
            Self::Name => Self::Bpm,
            Self::Bpm => Self::Name,
        }
    }
}

// ── Application state ───────────────────────────────────────────

pub struct App {
    pub project: Project,
    pub focus: Focus,
    pub voice_index: usize,
    pub editor_field: EditorField,
    pub header_field: HeaderField,
    /// Whether we are in text‑input mode (for pattern / name / bpm editing).
    pub editing: bool,
    /// Cursor position inside the text field being edited.
    pub cursor: usize,
    /// The text buffer for the field being edited.
    pub input_buf: String,
    /// Status message shown at the bottom.
    pub status: String,
    /// Is the track currently being played?
    pub playing: bool,
    /// Audio engine handle.
    pub engine: Option<AudioEngine>,
    /// Blink timer for cursor.
    pub blink_on: bool,
    blink_timer: Instant,
    /// Should the app quit?
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        let engine = AudioEngine::try_new();
        Ok(Self {
            project: Project::new(),
            focus: Focus::VoiceList,
            voice_index: 0,
            editor_field: EditorField::Pattern,
            header_field: HeaderField::Name,
            editing: false,
            cursor: 0,
            input_buf: String::new(),
            status: String::from("Bienvenue ! Appuyez sur ? pour l'aide."),
            playing: false,
            engine,
            blink_on: true,
            blink_timer: Instant::now(),
            should_quit: false,
        })
    }

    // ── Main loop ───────────────────────────────────────────────

    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Guard RAII : restaure toujours le terminal, même en cas de panic
        // ou d'erreur dans la boucle principale.
        let result = self.main_loop(&mut terminal);

        // Restauration inconditionnelle du terminal
        let _ = disable_raw_mode();
        let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
        let _ = terminal.show_cursor();

        result
    }

    /// Boucle principale isolée pour garantir la restauration du terminal
    /// via le guard RAII dans `run()`.
    fn main_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        let tick_rate = Duration::from_millis(80);

        loop {
            // Blink cursor every 500ms
            if self.blink_timer.elapsed() > Duration::from_millis(500) {
                self.blink_on = !self.blink_on;
                self.blink_timer = Instant::now();
            }

            terminal.draw(|f| ui::draw(f, self))?;

            if event::poll(tick_rate)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    self.handle_key(key.code, key.modifiers);
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    // ── Key handling ────────────────────────────────────────────

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        // Universal shortcuts (always active)
        if modifiers.contains(KeyModifiers::CONTROL) {
            match code {
                KeyCode::Char('q') => {
                    self.should_quit = true;
                    return;
                }
                KeyCode::Char('s') => {
                    self.save_project();
                    return;
                }
                KeyCode::Char('o') => {
                    self.load_last_project();
                    return;
                }
                KeyCode::Char('p') | KeyCode::Char(' ') => {
                    self.toggle_play();
                    return;
                }
                _ => {}
            }
        }

        if self.editing {
            self.handle_editing_key(code);
        } else {
            self.handle_navigation_key(code, modifiers);
        }
    }

    fn handle_editing_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                // Cancel editing, revert buffer.
                self.editing = false;
                self.status = String::from("Édition annulée.");
            }
            KeyCode::Enter => {
                self.commit_edit();
                self.editing = false;
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.input_buf.remove(self.cursor);
                }
            }
            KeyCode::Delete => {
                if self.cursor < self.input_buf.len() {
                    self.input_buf.remove(self.cursor);
                }
            }
            KeyCode::Left => {
                self.cursor = self.cursor.saturating_sub(1);
            }
            KeyCode::Right => {
                if self.cursor < self.input_buf.len() {
                    self.cursor += 1;
                }
            }
            KeyCode::Home => {
                self.cursor = 0;
            }
            KeyCode::End => {
                self.cursor = self.input_buf.len();
            }
            KeyCode::Char(c) => {
                self.input_buf.insert(self.cursor, c);
                self.cursor += 1;
            }
            _ => {}
        }
    }

    fn handle_navigation_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        match code {
            // Quit
            KeyCode::Char('q') => self.should_quit = true,

            // Focus switching
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::TrackHeader => Focus::VoiceList,
                    Focus::VoiceList => Focus::Editor,
                    Focus::Editor => Focus::TrackHeader,
                };
            }
            KeyCode::BackTab => {
                self.focus = match self.focus {
                    Focus::TrackHeader => Focus::Editor,
                    Focus::VoiceList => Focus::TrackHeader,
                    Focus::Editor => Focus::VoiceList,
                };
            }

            // Navigation within current focus
            KeyCode::Up | KeyCode::Char('k') => self.navigate_up(),
            KeyCode::Down | KeyCode::Char('j') => self.navigate_down(),
            KeyCode::Left | KeyCode::Char('h') => self.navigate_left(),
            KeyCode::Right | KeyCode::Char('l') => self.navigate_right(),

            // Enter editing mode / confirm selection
            KeyCode::Enter | KeyCode::Char('e') => self.start_edit(),

            // Add / remove voices
            KeyCode::Char('a') => self.add_voice(),
            KeyCode::Char('d') => self.delete_voice(),
            KeyCode::Char('D') => self.duplicate_voice(),

            // Play / stop
            KeyCode::Char(' ') => self.toggle_play(),

            // Save / load
            KeyCode::Char('s') if modifiers.is_empty() => self.save_project(),

            // Help
            KeyCode::Char('?') => {
                self.status = String::from(
                    "Tab: focus | j/k: nav | Enter/e: éditer | a: ajouter voix | d: supprimer | D: dupliquer | Espace: play | Ctrl+S: sauver | q: quitter",
                );
            }

            _ => {}
        }
    }

    // ── Navigation helpers ──────────────────────────────────────

    fn navigate_up(&mut self) {
        match self.focus {
            Focus::VoiceList => {
                self.voice_index = self.voice_index.saturating_sub(1);
            }
            Focus::Editor => {
                self.editor_field = self.editor_field.prev();
            }
            Focus::TrackHeader => {
                self.header_field = self.header_field.next();
            }
        }
    }

    fn navigate_down(&mut self) {
        match self.focus {
            Focus::VoiceList => {
                if !self.project.voices.is_empty() {
                    self.voice_index = (self.voice_index + 1).min(self.project.voices.len() - 1);
                }
            }
            Focus::Editor => {
                self.editor_field = self.editor_field.next();
            }
            Focus::TrackHeader => {
                self.header_field = self.header_field.next();
            }
        }
    }

    fn navigate_left(&mut self) {
        match self.focus {
            Focus::Editor => {
                if self.editor_field == EditorField::Waveform {
                    self.cycle_waveform(false);
                } else if self.editor_field == EditorField::Amplitude {
                    self.adjust_amplitude(-0.01);
                } else if self.editor_field == EditorField::IsDrum {
                    self.toggle_drum();
                }
            }
            Focus::TrackHeader if self.header_field == HeaderField::Bpm => {
                self.project.bpm = (self.project.bpm - 1.0).max(20.0);
            }
            _ => {}
        }
    }

    fn navigate_right(&mut self) {
        match self.focus {
            Focus::Editor => {
                if self.editor_field == EditorField::Waveform {
                    self.cycle_waveform(true);
                } else if self.editor_field == EditorField::Amplitude {
                    self.adjust_amplitude(0.01);
                } else if self.editor_field == EditorField::IsDrum {
                    self.toggle_drum();
                }
            }
            Focus::TrackHeader if self.header_field == HeaderField::Bpm => {
                self.project.bpm = (self.project.bpm + 1.0).min(300.0);
            }
            _ => {}
        }
    }

    // ── Editing ─────────────────────────────────────────────────

    fn start_edit(&mut self) {
        match self.focus {
            Focus::Editor => {
                if self.project.voices.is_empty() {
                    return;
                }
                let voice = &self.project.voices[self.voice_index];
                match self.editor_field {
                    EditorField::Pattern => {
                        self.input_buf = voice.pattern.clone();
                        self.cursor = self.input_buf.len();
                        self.editing = true;
                    }
                    EditorField::Waveform => {
                        self.cycle_waveform(true);
                    }
                    EditorField::Amplitude => {
                        self.input_buf = format!("{:.2}", voice.amplitude);
                        self.cursor = self.input_buf.len();
                        self.editing = true;
                    }
                    EditorField::IsDrum => {
                        self.toggle_drum();
                    }
                }
            }
            Focus::TrackHeader => match self.header_field {
                HeaderField::Name => {
                    self.input_buf = self.project.name.clone();
                    self.cursor = self.input_buf.len();
                    self.editing = true;
                }
                HeaderField::Bpm => {
                    self.input_buf = format!("{:.0}", self.project.bpm);
                    self.cursor = self.input_buf.len();
                    self.editing = true;
                }
            },
            Focus::VoiceList => {
                // Switch to editor for the selected voice.
                self.focus = Focus::Editor;
                self.editor_field = EditorField::Pattern;
            }
        }
    }

    fn commit_edit(&mut self) {
        match self.focus {
            Focus::Editor => {
                if self.voice_index >= self.project.voices.len() {
                    return;
                }
                let voice = &mut self.project.voices[self.voice_index];
                match self.editor_field {
                    EditorField::Pattern => {
                        voice.pattern = self.input_buf.clone();
                        self.status = String::from("Pattern mis à jour.");
                    }
                    EditorField::Amplitude => {
                        if let Ok(val) = self.input_buf.parse::<f32>() {
                            voice.amplitude = val.clamp(0.0, 1.0);
                            self.status = format!("Amplitude: {:.2}", voice.amplitude);
                        } else {
                            self.status = String::from("Valeur invalide.");
                        }
                    }
                    _ => {}
                }
            }
            Focus::TrackHeader => match self.header_field {
                HeaderField::Name => {
                    self.project.name = self.input_buf.clone();
                    self.status = format!("Nom du track: {}", self.project.name);
                }
                HeaderField::Bpm => {
                    if let Ok(val) = self.input_buf.parse::<f64>() {
                        self.project.bpm = val.clamp(20.0, 300.0);
                        self.status = format!("BPM: {:.0}", self.project.bpm);
                    } else {
                        self.status = String::from("BPM invalide.");
                    }
                }
            },
            _ => {}
        }

        // If playing, refresh preview.
        if self.playing {
            self.play_preview();
        }
    }

    // ── Voice management ────────────────────────────────────────

    fn add_voice(&mut self) {
        self.project.voices.push(VoiceDef {
            pattern: String::from("c4 e4 g4 c5"),
            waveform: String::from("sine"),
            amplitude: 0.15,
            is_drum: false,
        });
        self.voice_index = self.project.voices.len() - 1;
        self.status = format!("Voix {} ajoutée.", self.project.voices.len());
    }

    fn delete_voice(&mut self) {
        if self.project.voices.is_empty() {
            return;
        }
        self.project.voices.remove(self.voice_index);
        if self.voice_index > 0 && self.voice_index >= self.project.voices.len() {
            self.voice_index -= 1;
        }
        self.status = String::from("Voix supprimée.");
    }

    fn duplicate_voice(&mut self) {
        if self.project.voices.is_empty() {
            return;
        }
        let clone = self.project.voices[self.voice_index].clone();
        self.project.voices.insert(self.voice_index + 1, clone);
        self.voice_index += 1;
        self.status = String::from("Voix dupliquée.");
    }

    fn cycle_waveform(&mut self, forward: bool) {
        if self.project.voices.is_empty() {
            return;
        }
        let voice = &mut self.project.voices[self.voice_index];
        let current = WAVEFORMS
            .iter()
            .position(|(name, _)| *name == voice.waveform)
            .unwrap_or(0);
        let next = if forward {
            (current + 1) % WAVEFORMS.len()
        } else {
            (current + WAVEFORMS.len() - 1) % WAVEFORMS.len()
        };
        voice.waveform = WAVEFORMS[next].0.to_string();
        self.status = format!("Waveform: {}", voice.waveform);
    }

    fn adjust_amplitude(&mut self, delta: f32) {
        if self.project.voices.is_empty() {
            return;
        }
        let voice = &mut self.project.voices[self.voice_index];
        voice.amplitude = (voice.amplitude + delta).clamp(0.0, 1.0);
        self.status = format!("Amplitude: {:.2}", voice.amplitude);
    }

    fn toggle_drum(&mut self) {
        if self.project.voices.is_empty() {
            return;
        }
        let voice = &mut self.project.voices[self.voice_index];
        voice.is_drum = !voice.is_drum;
        self.status = format!("Drum: {}", if voice.is_drum { "oui" } else { "non" });
    }

    // ── Playback ────────────────────────────────────────────────

    fn toggle_play(&mut self) {
        if self.playing {
            self.stop_preview();
        } else {
            self.play_preview();
        }
    }

    fn play_preview(&mut self) {
        if let Some(engine) = &self.engine {
            let track = self.project.to_track();
            engine.play_track(&track);
            self.playing = true;
            self.status = format!(
                "▶ Lecture: {} @ {} BPM",
                self.project.name, self.project.bpm
            );
        } else {
            self.status = String::from("Pas de périphérique audio disponible.");
        }
    }

    fn stop_preview(&mut self) {
        if let Some(engine) = &self.engine {
            engine.stop_music();
        }
        self.playing = false;
        self.status = String::from("■ Arrêté.");
    }

    // ── Save / Load ─────────────────────────────────────────────

    fn save_project(&mut self) {
        let dir = project::projects_dir();
        let filename = format!("{}.json", self.project.name);
        let path = dir.join(&filename);
        match self.project.save(&path) {
            Ok(()) => {
                self.status = format!("Sauvegardé: {}", path.display());
            }
            Err(e) => {
                self.status = format!("Erreur de sauvegarde: {e}");
            }
        }
    }

    fn load_last_project(&mut self) {
        let files = project::list_projects();
        if let Some(path) = files.last() {
            match Project::load(path) {
                Ok(p) => {
                    self.project = p;
                    self.voice_index = 0;
                    self.status = format!("Chargé: {}", path.display());
                }
                Err(e) => {
                    self.status = format!("Erreur de chargement: {e}");
                }
            }
        } else {
            self.status = String::from("Aucun projet trouvé.");
        }
    }

    // ── Public accessors ────────────────────────────────────────

    /// Currently selected voice (if any).
    pub fn current_voice(&self) -> Option<&VoiceDef> {
        self.project.voices.get(self.voice_index)
    }
}
