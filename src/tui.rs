use crate::app::{App, AppState};
use crate::visualizer::Visualizer;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Padding, Paragraph},
    Frame,
};

const MAX_WIDTH: u16 = 80;
const MIN_WIDTH: u16 = 40;

const COLOR_TITLE: Color = Color::Green;
const COLOR_ACCENT: Color = Color::Yellow;
const COLOR_DIM: Color = Color::DarkGray;
const COLOR_TEXT: Color = Color::White;
const COLOR_PLAYING: Color = Color::Green;
const COLOR_VOL_FILL: Color = Color::Green;
const COLOR_VOL_EMPTY: Color = Color::DarkGray;

pub fn draw(frame: &mut Frame, app: &App, vis: &mut Visualizer) {
    let area = frame.area();

    // Responsive width: fill terminal but clamp to MIN..MAX
    let frame_w = area.width.clamp(MIN_WIDTH, MAX_WIDTH).min(area.width);
    let pad_left = area.width.saturating_sub(frame_w) / 2;
    let pad_top = area.height.saturating_sub(area.height) / 2; // 0, use full height

    let centered = Rect::new(area.x + pad_left, area.y + pad_top, frame_w, area.height);

    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(COLOR_DIM))
        .padding(Padding::new(2, 2, 1, 0));

    let inner = outer_block.inner(centered);
    frame.render_widget(outer_block, centered);

    let w = inner.width as usize;

    // Fixed-height sections + flexible spectrum
    let layer_count = app.engine.layers.len() as u16;
    let fixed_lines: u16 = 2      // title + scene
        + 1                        // blank
        + 1                        // blank after spectrum
        + layer_count              // layer volumes
        + 1                        // blank
        + 1                        // master volume
        + 1                        // blank
        + 1;                       // help bar

    let vis_height = inner.height.saturating_sub(fixed_lines).max(2);

    let constraints = vec![
        Constraint::Length(2),           // title + scene
        Constraint::Length(1),           // blank
        Constraint::Length(vis_height),  // spectrum (fills remaining)
        Constraint::Length(1),           // blank
        Constraint::Length(layer_count), // layer volumes
        Constraint::Length(1),           // blank
        Constraint::Length(1),           // master volume
        Constraint::Min(0),             // spacer pushes help to bottom
        Constraint::Length(1),           // help bar
    ];

    let chunks = Layout::vertical(constraints).split(inner);

    // ── Title + scene ──
    render_header(frame, app, chunks[0], w);

    // ── Spectrum ──
    let vis_h = chunks[2].height as usize;
    let vis_lines = vis.render(w, vis_h);
    frame.render_widget(Paragraph::new(vis_lines), chunks[2]);

    // ── Layer volumes ──
    render_layers(frame, app, chunks[4], w);

    // ── Master volume ──
    render_master(frame, app, chunks[6], w);

    // ── Help bar ──
    render_help(frame, chunks[8]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect, width: usize) {
    let state_span = match &app.state {
        AppState::Loading { done, total } => Span::styled(
            format!("⏳ Loading {done}/{total}"),
            Style::default().fg(COLOR_ACCENT),
        ),
        AppState::Playing => Span::styled(
            "▶ Playing".to_string(),
            Style::default()
                .fg(COLOR_PLAYING)
                .add_modifier(Modifier::BOLD),
        ),
        AppState::Paused => Span::styled(
            "⏸ Paused".to_string(),
            Style::default()
                .fg(COLOR_ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
    };

    let title_text = "V I B E B A N D";
    let state_len = match &app.state {
        AppState::Loading { done, total } => format!("⏳ Loading {done}/{total}").len(),
        AppState::Playing => "▶ Playing".len(),
        AppState::Paused => "⏸ Paused".len(),
    };
    let gap = width.saturating_sub(title_text.len() + state_len);

    let title_line = Line::from(vec![
        Span::styled(
            title_text.to_string(),
            Style::default()
                .fg(COLOR_TITLE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" ".repeat(gap)),
        state_span,
    ]);

    // Scene names
    let scene_list: Vec<String> = app
        .engine
        .layers
        .iter()
        .map(|l| format!("{} {}", l.emoji, l.label))
        .collect();

    let scene_line = if !scene_list.is_empty() {
        Line::from(Span::styled(
            format!("♫ {}", scene_list.join(" + ")),
            Style::default().fg(COLOR_ACCENT),
        ))
    } else {
        Line::from(Span::styled(
            "♫ Loading scenes...".to_string(),
            Style::default().fg(COLOR_DIM),
        ))
    };

    frame.render_widget(Paragraph::new(vec![title_line, scene_line]), area);
}

fn render_layers(frame: &mut Frame, app: &App, area: Rect, width: usize) {
    let mut lines = Vec::new();

    for (i, layer) in app.engine.layers.iter().enumerate() {
        let selected =
            i == app.selected_layer && !matches!(app.state, AppState::Loading { .. });

        let prefix = if selected { "▸ " } else { "  " };
        let pct = (layer.volume * 100.0) as u32;

        let label = format!("{}{} {}", prefix, layer.emoji, layer.label);
        let vol_str = format!(" {:>3}%", pct);

        let bar_total = width.saturating_sub(label.len() + vol_str.len() + 3);
        let filled = ((layer.volume as f64) * bar_total as f64) as usize;
        let empty = bar_total.saturating_sub(filled);

        let label_color = if selected { COLOR_ACCENT } else { COLOR_TEXT };

        lines.push(Line::from(vec![
            Span::styled(label, Style::default().fg(label_color)),
            Span::raw(" "),
            Span::styled("█".repeat(filled), Style::default().fg(COLOR_VOL_FILL)),
            Span::styled("░".repeat(empty), Style::default().fg(COLOR_VOL_EMPTY)),
            Span::styled(vol_str, Style::default().fg(COLOR_DIM)),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_master(frame: &mut Frame, app: &App, area: Rect, width: usize) {
    let master_pct = (app.engine.master_volume * 100.0) as u32;
    let label = "VOL ";
    let vol_str = format!(" {:>3}%", master_pct);
    let bar_total = width.saturating_sub(label.len() + vol_str.len() + 1);
    let filled = ((app.engine.master_volume as f64) * bar_total as f64) as usize;
    let empty = bar_total.saturating_sub(filled);

    let line = Line::from(vec![
        Span::styled(
            label.to_string(),
            Style::default()
                .fg(COLOR_TEXT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("█".repeat(filled), Style::default().fg(COLOR_VOL_FILL)),
        Span::styled("░".repeat(empty), Style::default().fg(COLOR_VOL_EMPTY)),
        Span::styled(vol_str, Style::default().fg(COLOR_DIM)),
    ]);

    frame.render_widget(Paragraph::new(vec![line]), area);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled("[Spc]", Style::default().fg(COLOR_ACCENT)),
        Span::styled("⏯  ", Style::default().fg(COLOR_DIM)),
        Span::styled("[↑↓]", Style::default().fg(COLOR_ACCENT)),
        Span::styled("Vol ", Style::default().fg(COLOR_DIM)),
        Span::styled("[+-]", Style::default().fg(COLOR_ACCENT)),
        Span::styled("Master ", Style::default().fg(COLOR_DIM)),
        Span::styled("[Tab]", Style::default().fg(COLOR_ACCENT)),
        Span::styled("Layer ", Style::default().fg(COLOR_DIM)),
        Span::styled("[Q]", Style::default().fg(COLOR_ACCENT)),
        Span::styled("Quit", Style::default().fg(COLOR_DIM)),
    ]);

    frame.render_widget(Paragraph::new(vec![line]), area);
}
