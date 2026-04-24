use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let entry_name = app
        .units
        .selected_entry()
        .map(|e| e.name.as_str())
        .unwrap_or("unknown");

    let lines: Vec<Line> = if app.unit_file_cache.is_empty() {
        vec![Line::from(Span::styled(
            "No unit file found (unit may be transient or not installed)",
            Style::default().fg(theme.text_dim),
        ))]
    } else {
        app.unit_file_cache
            .iter()
            .map(|l| {
                // Highlight section headers like [Unit], [Service], etc.
                if l.starts_with('[') && l.ends_with(']') {
                    Line::from(Span::styled(
                        l.clone(),
                        Style::default()
                            .fg(theme.header)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else if let Some((key, val)) = l.split_once('=') {
                    Line::from(vec![
                        Span::styled(
                            format!("{}=", key),
                            Style::default().fg(theme.text_dim),
                        ),
                        Span::styled(val.to_string(), Style::default().fg(theme.text)),
                    ])
                } else {
                    Line::from(Span::styled(
                        l.clone(),
                        Style::default().fg(theme.text_dim),
                    ))
                }
            })
            .collect()
    };

    let widget = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border_focused))
                .title(Span::styled(
                    format!(" Unit File: {} (q/esc to close) ", entry_name),
                    Style::default()
                        .fg(theme.header)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .scroll((app.unit_file_scroll as u16, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(widget, area);
}
