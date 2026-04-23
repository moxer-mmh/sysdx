use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState},
    Frame,
};

use crate::systemd::Action;

pub fn render(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = centered_rect(30, 60, frame.area());

    let items: Vec<ListItem> = Action::all()
        .iter()
        .map(|a| ListItem::new(Span::styled(a.label(), Style::default().fg(theme.text))))
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.action_menu_selected));

    let unit_name = app
        .units
        .selected_entry()
        .map(|e| e.name.as_str())
        .unwrap_or("");

    let title = format!(" {} ", unit_name);

    // Clear the area behind the popup so it doesn't bleed through
    frame.render_widget(Clear, area);

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border_focused))
                .title(Span::styled(
                    title,
                    Style::default()
                        .fg(theme.header)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .highlight_style(
            Style::default()
                .bg(theme.selection_bg)
                .fg(theme.selection_fg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut state);
}

fn centered_rect(width_pct: u16, height_pct: u16, area: Rect) -> Rect {
    let h = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_pct) / 2),
            Constraint::Percentage(height_pct),
            Constraint::Percentage((100 - height_pct) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_pct) / 2),
            Constraint::Percentage(width_pct),
            Constraint::Percentage((100 - width_pct) / 2),
        ])
        .split(h[1])[1]
}
