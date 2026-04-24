use crate::app::{App, Mode};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App, props_area: Rect, journal_area: Rect) {
    let theme = &app.theme;
    let entry = app.units.selected_entry();

    let props_lines: Vec<Line> = if let Some(e) = entry {
        let label = |k: &str| {
            Span::styled(
                format!("{:<12}", k),
                Style::default()
                    .fg(theme.header)
                    .add_modifier(Modifier::BOLD),
            )
        };
        let val = |v: &str, color: ratatui::style::Color| {
            Span::styled(v.to_string(), Style::default().fg(color))
        };

        let active_color = match e.active_state.as_str() {
            "active" => theme.active,
            "failed" => theme.failed,
            _ => theme.inactive,
        };

        vec![
            Line::from(vec![label("Unit"), val(&e.name, theme.text)]),
            Line::from(vec![
                label("Description"),
                val(&e.description, theme.text_dim),
            ]),
            Line::from(vec![label("Load"), val(&e.load_state, theme.text_dim)]),
            Line::from(vec![
                label("Active"),
                val(&e.active_state, active_color),
                Span::raw(" ("),
                val(&e.sub_state, theme.text_dim),
                Span::raw(")"),
            ]),
            Line::from(vec![label("Type"), val(&e.unit_type, theme.text_dim)]),
        ]
    } else {
        vec![Line::from(Span::styled(
            "No unit selected",
            Style::default().fg(theme.text_dim),
        ))]
    };

    let props_widget = Paragraph::new(props_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border))
                .title(Span::styled(
                    " Status ",
                    Style::default()
                        .fg(theme.header)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(props_widget, props_area);

    let journal_lines: Vec<Line> = if app.status_cache.is_empty() {
        vec![Line::from(Span::styled(
            "No output",
            Style::default().fg(theme.text_dim),
        ))]
    } else {
        app.status_cache
            .lines()
            .map(|l| {
                Line::from(Span::styled(
                    l.to_string(),
                    Style::default().fg(theme.text_dim),
                ))
            })
            .collect()
    };

    let journal_widget = Paragraph::new(journal_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border))
                .title(Span::styled(
                    " Output ",
                    Style::default()
                        .fg(theme.header)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(journal_widget, journal_area);
}

pub fn render_log_view(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let entry_name = app
        .units
        .selected_entry()
        .map(|e| e.name.as_str())
        .unwrap_or("unknown");

    let live_indicator = if app.log_live { " ● LIVE" } else { "" };
    let title = format!(
        " Logs: {}{} (f live-tail  q/esc close) ",
        entry_name, live_indicator
    );

    let lines: Vec<Line> = if app.journal_cache.is_empty() {
        vec![Line::from(Span::styled(
            "No journal entries found",
            Style::default().fg(theme.text_dim),
        ))]
    } else {
        app.journal_cache
            .iter()
            .map(|l| Line::from(Span::styled(l.clone(), Style::default().fg(theme.text))))
            .collect()
    };

    let border_color = if app.log_live {
        theme.active
    } else {
        theme.border_focused
    };

    let widget = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    title,
                    Style::default()
                        .fg(theme.header)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .scroll((app.log_scroll as u16, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(widget, area);

    let _ = Mode::LogView; // suppress unused import warning
}
