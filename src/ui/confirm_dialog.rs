use crate::app::App;
use crate::systemd::Action;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = centered_rect(40, 7, frame.area());

    let pending_idx = match app.confirm_pending {
        Some(i) => i,
        None => return,
    };
    let action = &Action::all()[pending_idx];
    let unit_name = app
        .units
        .selected_entry()
        .map(|e| e.name.as_str())
        .unwrap_or("");

    let lines = vec![
        Line::from(vec![
            Span::styled("Action:  ", Style::default().fg(theme.text_dim)),
            Span::styled(action.label(), Style::default().fg(theme.failed).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Unit:    ", Style::default().fg(theme.text_dim)),
            Span::styled(unit_name, Style::default().fg(theme.text)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  y  confirm      n / esc  cancel",
            Style::default().fg(theme.header).add_modifier(Modifier::BOLD),
        )),
    ];

    frame.render_widget(Clear, area);

    let widget = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.failed))
                .title(Span::styled(
                    " Confirm ",
                    Style::default().fg(theme.failed).add_modifier(Modifier::BOLD),
                )),
        );

    frame.render_widget(widget, area);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let h = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(height),
            Constraint::Fill(1),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(width),
            Constraint::Fill(1),
        ])
        .split(h[1])[1]
}
