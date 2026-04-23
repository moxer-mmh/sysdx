use crate::{app::App, app::Mode};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App, list_area: Rect, filter_area: Option<Rect>) {
    let theme = &app.theme;
    let scope_label = app.units.scope.label();

    // Render filter bar when filtering
    if let Some(farea) = filter_area {
        let filter_text = format!(" / {}_", app.filter_query);
        let filter_widget = Paragraph::new(filter_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme.filter_bar))
                    .title(Span::styled(
                        " Filter ",
                        Style::default().fg(theme.filter_bar),
                    )),
            )
            .style(Style::default().fg(theme.filter_bar));
        frame.render_widget(filter_widget, farea);
    }

    let title = format!(
        " sysdx [{}] ({}/{}) ",
        scope_label,
        app.visible_indices.len(),
        app.units.entries.len(),
    );

    let border_style = if app.mode == Mode::Normal || app.mode == Mode::Filtering {
        Style::default().fg(theme.border_focused)
    } else {
        Style::default().fg(theme.border)
    };

    let items: Vec<ListItem> = app
        .visible_indices
        .iter()
        .map(|&i| {
            let entry = &app.units.entries[i];
            let indicator_color = match entry.active_state.as_str() {
                "active" => theme.active,
                "failed" => theme.failed,
                _ => theme.inactive,
            };
            let indicator = Span::styled(
                format!("{} ", entry.status_indicator()),
                Style::default().fg(indicator_color),
            );
            let name_color = if entry.active_state == "active" {
                theme.text
            } else {
                theme.text_dim
            };
            let name = Span::styled(
                if app.config.display.show_description && !entry.description.is_empty() {
                    format!("{:<35} {}", entry.name, entry.description)
                } else {
                    entry.name.clone()
                },
                Style::default().fg(name_color),
            );
            ListItem::new(Line::from(vec![indicator, name]))
        })
        .collect();

    // Find position of selected in visible list for ListState
    let list_pos = app
        .visible_indices
        .iter()
        .position(|&i| i == app.units.selected);

    let mut list_state = ListState::default();
    list_state.select(list_pos);

    let list_widget = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style)
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

    frame.render_stateful_widget(list_widget, list_area, &mut list_state);
}
