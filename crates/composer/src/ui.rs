//! UI rendering for the composer TUI.
//!
//! Features:
//! - Header: track name, BPM, voice count, play indicator
//! - Body: voice list (left) + editor panel (right)
//! - Timeline: multi‑lane overview of all voices with playback cursor
//! - Status bar: editing hints + messages

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use monster_battle_audio::pattern;
use monster_battle_audio::track_def::VoiceDef;

use crate::app::{App, EditorField, Focus, HeaderField};

// ── Colours ─────────────────────────────────────────────────────

const ACCENT: Color = Color::Cyan;
const FOCUSED_BORDER: Color = Color::Yellow;
const DIM: Color = Color::DarkGray;
const PLAYING: Color = Color::Green;
const DRUM_COLOR: Color = Color::Magenta;
const CURSOR_COLOR: Color = Color::White;

fn border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(FOCUSED_BORDER)
    } else {
        Style::default().fg(DIM)
    }
}

// ── Main draw ───────────────────────────────────────────────────

pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Top‑level layout: header (3) / body / timeline / status bar (3)
    let timeline_height = timeline_lanes_height(app);
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),              // header
            Constraint::Min(8),                 // body (editor + voice list)
            Constraint::Length(timeline_height), // timeline
            Constraint::Length(3),              // status bar
        ])
        .split(size);

    draw_header(frame, app, outer[0]);
    draw_body(frame, app, outer[1]);
    draw_timeline(frame, app, outer[2]);
    draw_status_bar(frame, app, outer[3]);
}

/// Compute the height needed for the timeline panel.
fn timeline_lanes_height(app: &App) -> u16 {
    let lanes = app.project.voices.len() as u16;
    // 2 for borders + 1 header line + 1 per voice, min 4
    (lanes + 3).max(4).min(14)
}

// ── Header: track name + BPM ────────────────────────────────────

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::TrackHeader;
    let block = Block::default()
        .title(" 🎵 Monster Battle Composer ")
        .borders(Borders::ALL)
        .border_style(border_style(focused));

    let play_indicator = if app.playing {
        Span::styled(
            " ▶ ",
            Style::default().fg(PLAYING).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(" ■ ", Style::default().fg(DIM))
    };

    let name_style = if focused && app.header_field == HeaderField::Name {
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let bpm_style = if focused && app.header_field == HeaderField::Bpm {
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let name_text = if app.editing && focused && app.header_field == HeaderField::Name {
        format!("{}", app.input_buf)
    } else {
        app.project.name.clone()
    };

    let bpm_text = if app.editing && focused && app.header_field == HeaderField::Bpm {
        format!("{}", app.input_buf)
    } else {
        format!("{:.0}", app.project.bpm)
    };

    let line = Line::from(vec![
        play_indicator,
        Span::styled("Nom: ", Style::default().fg(DIM)),
        Span::styled(name_text, name_style),
        Span::raw("  │  "),
        Span::styled("BPM: ", Style::default().fg(DIM)),
        Span::styled(bpm_text, bpm_style),
        Span::raw(format!("  │  Voix: {}", app.project.voices.len())),
    ]);

    let para = Paragraph::new(line).block(block);
    frame.render_widget(para, area);
}

// ── Body: voice list + editor ───────────────────────────────────

fn draw_body(frame: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    draw_voice_list(frame, app, cols[0]);
    draw_editor(frame, app, cols[1]);
}

// ── Voice list panel ────────────────────────────────────────────

fn draw_voice_list(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::VoiceList;
    let block = Block::default()
        .title(" Voix ")
        .borders(Borders::ALL)
        .border_style(border_style(focused));

    if app.project.voices.is_empty() {
        let msg = Paragraph::new("  (aucune voix)\n  Appuyez sur 'a' pour ajouter")
            .style(Style::default().fg(DIM))
            .block(block);
        frame.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = app
        .project
        .voices
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let marker = if i == app.voice_index && focused {
                "▸ "
            } else {
                "  "
            };
            let drum_tag = if v.is_drum { " 🥁" } else { "" };
            let selected_style = if i == app.voice_index {
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::raw(marker),
                Span::styled(format!("V{} ", i + 1), selected_style),
                Span::styled(
                    &v.waveform,
                    Style::default().fg(waveform_color(&v.waveform)),
                ),
                Span::styled(drum_tag, Style::default().fg(DRUM_COLOR)),
            ]))
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn waveform_color(name: &str) -> Color {
    match name {
        "sine" => Color::Blue,
        "square" => Color::Red,
        "sawtooth" => Color::Yellow,
        "triangle" => Color::Green,
        _ => Color::White,
    }
}

// ── Editor panel ────────────────────────────────────────────────

fn draw_editor(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Editor;
    let block = Block::default()
        .title(" Éditeur ")
        .borders(Borders::ALL)
        .border_style(border_style(focused));

    let Some(voice) = app.current_voice() else {
        let msg = Paragraph::new("  Sélectionnez ou ajoutez une voix.")
            .style(Style::default().fg(DIM))
            .block(block);
        frame.render_widget(msg, area);
        return;
    };

    // Split editor area: fields + pattern preview (for selected voice)
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // pattern
            Constraint::Length(1), // waveform
            Constraint::Length(1), // amplitude
            Constraint::Length(1), // is_drum
            Constraint::Length(1), // spacer
            Constraint::Min(3),   // pattern preview (single voice)
        ])
        .split(inner);

    // Pattern field (with text cursor when editing)
    let pat_focused = focused && app.editor_field == EditorField::Pattern;
    let pat_style = field_style(pat_focused);

    if app.editing && pat_focused {
        // Show text cursor inside the pattern field
        let before: String = app.input_buf[..app.cursor].to_string();
        let cursor_char = if app.blink_on { "│" } else { " " };
        let after: String = app.input_buf[app.cursor..].to_string();

        let pat_label = Line::from(vec![
            Span::styled(" Pattern: ", Style::default().fg(DIM)),
            Span::styled(format!(" {}", before), pat_style),
            Span::styled(
                cursor_char,
                Style::default()
                    .fg(CURSOR_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(after, pat_style),
        ]);
        let pat_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(DIM));
        let pat = Paragraph::new(pat_label)
            .block(pat_block)
            .wrap(Wrap { trim: false });
        frame.render_widget(pat, rows[0]);
    } else {
        let pat_label = Line::from(vec![
            Span::styled(" Pattern: ", Style::default().fg(DIM)),
            Span::styled(format!(" {}", voice.pattern), pat_style),
        ]);
        let pat_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(DIM));
        let pat = Paragraph::new(pat_label)
            .block(pat_block)
            .wrap(Wrap { trim: false });
        frame.render_widget(pat, rows[0]);
    }

    // Waveform field
    let wf_focused = focused && app.editor_field == EditorField::Waveform;
    let wf_line = Line::from(vec![
        Span::styled(" Waveform: ", Style::default().fg(DIM)),
        Span::styled(
            format!("◀ {} ▶", voice.waveform),
            field_style(wf_focused).fg(waveform_color(&voice.waveform)),
        ),
    ]);
    frame.render_widget(Paragraph::new(wf_line), rows[1]);

    // Amplitude field (with text cursor when editing)
    let amp_focused = focused && app.editor_field == EditorField::Amplitude;
    let bar_width = (rows[2].width as f32 * 0.4) as usize;
    let filled = (voice.amplitude * bar_width as f32) as usize;
    let bar: String = "█".repeat(filled) + &"░".repeat(bar_width.saturating_sub(filled));

    if app.editing && amp_focused {
        let before: String = app.input_buf[..app.cursor].to_string();
        let cursor_char = if app.blink_on { "│" } else { " " };
        let after: String = app.input_buf[app.cursor..].to_string();
        let amp_line = Line::from(vec![
            Span::styled(" Amplitude: ", Style::default().fg(DIM)),
            Span::styled(before, field_style(true)),
            Span::styled(
                cursor_char,
                Style::default()
                    .fg(CURSOR_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(after, field_style(true)),
            Span::raw("  "),
            Span::styled(bar, Style::default().fg(ACCENT)),
        ]);
        frame.render_widget(Paragraph::new(amp_line), rows[2]);
    } else {
        let amp_text = format!("{:.2}", voice.amplitude);
        let amp_line = Line::from(vec![
            Span::styled(" Amplitude: ", Style::default().fg(DIM)),
            Span::styled(amp_text, field_style(amp_focused)),
            Span::raw("  "),
            Span::styled(bar, Style::default().fg(ACCENT)),
        ]);
        frame.render_widget(Paragraph::new(amp_line), rows[2]);
    }

    // IsDrum field
    let drum_focused = focused && app.editor_field == EditorField::IsDrum;
    let drum_text = if voice.is_drum { "oui 🥁" } else { "non" };
    let drum_line = Line::from(vec![
        Span::styled(" Drum: ", Style::default().fg(DIM)),
        Span::styled(drum_text, field_style(drum_focused)),
    ]);
    frame.render_widget(Paragraph::new(drum_line), rows[3]);

    // Single‑voice pattern preview
    draw_pattern_vis(frame, voice, app.playback_position(), rows[5]);
}

fn field_style(focused: bool) -> Style {
    if focused {
        Style::default()
            .fg(ACCENT)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        Style::default()
    }
}

// ── Pattern preview (single voice) ──────────────────────────────

fn draw_pattern_vis(
    frame: &mut Frame,
    voice: &VoiceDef,
    playback_pos: Option<f64>,
    area: Rect,
) {
    let block = Block::default()
        .title(" Aperçu (cycle 0) ")
        .borders(Borders::TOP)
        .border_style(Style::default().fg(DIM));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let pat = pattern::parse(&voice.pattern);
    let events = pat.query(0);
    let width = inner.width as usize;
    if width == 0 {
        return;
    }

    if events.is_empty() {
        let msg = Paragraph::new("  (silence)").style(Style::default().fg(DIM));
        frame.render_widget(msg, inner);
        return;
    }

    let color = waveform_color(&voice.waveform);

    // Build grid with events
    let mut grid: Vec<(char, Color)> = vec![('░', DIM); width];
    for ev in &events {
        let start_col = (ev.start * width as f64) as usize;
        let end_col = ((ev.start + ev.duration) * width as f64).ceil() as usize;
        let end_col = end_col.min(width);
        for col in start_col..end_col {
            grid[col] = ('█', color);
        }
    }

    // Overlay playback cursor
    if let Some(pos) = playback_pos {
        let cursor_col = (pos * width as f64) as usize;
        if cursor_col < width {
            grid[cursor_col] = ('▎', CURSOR_COLOR);
        }
    }

    let spans: Vec<Span> = grid
        .into_iter()
        .map(|(ch, c)| Span::styled(ch.to_string(), Style::default().fg(c)))
        .collect();
    let vis = Paragraph::new(Line::from(spans));
    frame.render_widget(vis, inner);
}

// ── Timeline: multi‑lane overview ───────────────────────────────

fn draw_timeline(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" 🎼 Timeline ")
        .borders(Borders::ALL)
        .border_style(border_style(false));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.project.voices.is_empty() || inner.width < 6 || inner.height < 1 {
        return;
    }

    let width = inner.width as usize;
    let playback_pos = app.playback_position();

    // Draw one lane per voice (up to available height)
    let max_lanes = inner.height as usize;
    let label_width: usize = 4; // "V1 " + separator
    let lane_width = width.saturating_sub(label_width + 1);

    if lane_width == 0 {
        return;
    }

    let mut lines: Vec<Line> = Vec::new();

    // Ruler line (beat markers)
    let mut ruler_spans = vec![Span::styled(
        format!("{:<width$}", "", width = label_width + 1),
        Style::default().fg(DIM),
    )];
    for col in 0..lane_width {
        let frac = col as f64 / lane_width as f64;
        let beat = (frac * 4.0) as usize;
        let is_beat_boundary = col == (beat as f64 * lane_width as f64 / 4.0) as usize;
        if is_beat_boundary && beat < 4 {
            ruler_spans.push(Span::styled(
                format!("{}", beat + 1),
                Style::default().fg(DIM),
            ));
        } else {
            ruler_spans.push(Span::styled("·", Style::default().fg(Color::Black)));
        }
    }
    lines.push(Line::from(ruler_spans));

    // Voice lanes
    for (i, voice) in app.project.voices.iter().enumerate() {
        if lines.len() >= max_lanes {
            break;
        }

        let is_selected = i == app.voice_index;
        let color = waveform_color(&voice.waveform);
        let pat = pattern::parse(&voice.pattern);
        let events = pat.query(0);

        // Build the lane grid
        let mut grid: Vec<(char, Color)> = vec![('·', Color::Black); lane_width];
        for ev in &events {
            let start_col = (ev.start * lane_width as f64) as usize;
            let end_col = ((ev.start + ev.duration) * lane_width as f64).ceil() as usize;
            let end_col = end_col.min(lane_width);
            for col in start_col..end_col {
                if voice.is_drum {
                    grid[col] = ('▮', DRUM_COLOR);
                } else {
                    grid[col] = ('█', color);
                }
            }
        }

        // Overlay playback cursor on this lane
        if let Some(pos) = playback_pos {
            let cursor_col = (pos * lane_width as f64) as usize;
            if cursor_col < lane_width {
                grid[cursor_col] = ('▎', CURSOR_COLOR);
            }
        }

        // Voice label
        let label_style = if is_selected {
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(DIM)
        };

        let mut spans = vec![
            Span::styled(format!("V{:<2}", i + 1), label_style),
            Span::styled("│", Style::default().fg(DIM)),
        ];

        // Lane cells
        for (ch, c) in &grid {
            spans.push(Span::styled(ch.to_string(), Style::default().fg(*c)));
        }

        lines.push(Line::from(spans));
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}

// ── Status bar ──────────────────────────────────────────────────

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM));

    let editing_hint = if app.editing {
        " [ÉDITION — Enter: valider, Esc: annuler] "
    } else {
        ""
    };

    let line = Line::from(vec![
        Span::styled(editing_hint, Style::default().fg(Color::Yellow)),
        Span::styled(&app.status, Style::default().fg(Color::White)),
    ]);

    let para = Paragraph::new(line).block(block);
    frame.render_widget(para, area);
}
