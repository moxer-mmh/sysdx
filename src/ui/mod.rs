mod action_menu;
mod confirm_dialog;
mod help;
mod layout;
mod status_panel;
mod unit_file;
mod unit_list;

use crate::app::{App, Mode};
use ratatui::{
    style::Style,
    text::Span,
    widgets::{Block, Paragraph},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Background fill
    frame.render_widget(
        Block::default().style(
            Style::default()
                .bg(app.theme.background)
                .fg(app.theme.surface),
        ),
        area,
    );

    // Carve out a 1-row status bar at the bottom
    let (main_area, bar_area) = layout::with_status_bar(area);

    // --- Full-screen modes ---
    if app.mode == Mode::LogView {
        status_panel::render_log_view(frame, app, main_area);
        render_status_bar(frame, app, bar_area);
        return;
    }

    if app.mode == Mode::UnitFile {
        unit_file::render(frame, app, main_area);
        render_status_bar(frame, app, bar_area);
        return;
    }

    // --- Normal two-pane layout ---
    let panes = layout::split(main_area, app.config.display.list_width_pct);

    let (list_area, filter_area) = if app.mode == Mode::Filtering {
        let (fbar, larea) = layout::with_filter_bar(panes.list);
        (larea, Some(fbar))
    } else {
        (panes.list, None)
    };

    unit_list::render(frame, app, list_area, filter_area);

    let (props_area, journal_area) = layout::status_split(panes.status);
    status_panel::render(frame, app, props_area, journal_area);

    // --- Overlays (rendered last, on top) ---
    if app.mode == Mode::ActionMenu {
        action_menu::render(frame, app);
    }

    if app.mode == Mode::Confirm {
        confirm_dialog::render(frame, app);
    }

    if app.mode == Mode::Help {
        help::render(frame, app);
    }

    render_status_bar(frame, app, bar_area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let text = app.status_bar_text();
    let color = if app.units.last_error.is_some() || app.status_message.as_ref().map(|m| m.starts_with("Error")).unwrap_or(false) {
        app.theme.failed
    } else if app.status_message.is_some() {
        app.theme.active
    } else {
        app.theme.text_dim
    };

    let widget = Paragraph::new(Span::styled(text, Style::default().fg(color)))
        .style(Style::default().bg(app.theme.surface));

    frame.render_widget(widget, area);
}
