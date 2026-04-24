use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = centered_rect(58, 26, frame.area());

    let kb = &app.config.keybinds;
    let key = |k: &str| {
        Span::styled(
            format!("{:<12}", k),
            Style::default()
                .fg(theme.header)
                .add_modifier(Modifier::BOLD),
        )
    };
    let desc = |d: &str| Span::styled(d.to_string(), Style::default().fg(theme.text));
    let section = |s: &str| {
        Line::from(Span::styled(
            s.to_string(),
            Style::default()
                .fg(theme.text_dim)
                .add_modifier(Modifier::BOLD),
        ))
    };

    let lines = vec![
        section("Navigation"),
        Line::from(vec![key(&kb.move_down), desc("Move down")]),
        Line::from(vec![key(&kb.move_up), desc("Move up")]),
        Line::from(vec![key(&kb.page_down), desc("Half-page down")]),
        Line::from(vec![key(&kb.page_up), desc("Half-page up")]),
        Line::from(vec![key(&kb.go_top), desc("Go to top")]),
        Line::from(vec![key(&kb.go_bottom), desc("Go to bottom")]),
        Line::from(""),
        section("Search & Filter"),
        Line::from(vec![key(&kb.filter), desc("Open fuzzy filter")]),
        Line::from(vec![
            key(&kb.type_filter),
            desc("Cycle type filter  (service/socket/timer…)"),
        ]),
        Line::from(""),
        section("Actions"),
        Line::from(vec![key(&kb.action_menu), desc("Open action menu")]),
        Line::from(vec![
            key(&kb.switch_scope),
            desc("Toggle user ↔ system scope"),
        ]),
        Line::from(vec![key(&kb.refresh), desc("Refresh unit list")]),
        Line::from(""),
        section("Views"),
        Line::from(vec![
            key(&kb.open_logs),
            desc("Journal logs  (f = live-tail)"),
        ]),
        Line::from(vec![
            key(&kb.open_unit_file),
            desc("Unit file  (systemctl cat)"),
        ]),
        Line::from(vec![key(&kb.help), desc("This help screen")]),
        Line::from(""),
        section("General"),
        Line::from(vec![key(&kb.quit), desc("Quit")]),
        Line::from(vec![key("esc"), desc("Close overlay / cancel")]),
    ];

    frame.render_widget(Clear, area);

    let widget = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border_focused))
            .title(Span::styled(
                " Keybinds ",
                Style::default()
                    .fg(theme.header)
                    .add_modifier(Modifier::BOLD),
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
