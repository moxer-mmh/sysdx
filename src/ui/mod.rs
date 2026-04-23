mod action_menu;
mod layout;
mod status_panel;
mod unit_list;

use crate::app::{App, Mode};
use ratatui::{style::Style, widgets::Block, Frame};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Fill background
    let bg = Block::default().style(
        Style::default()
            .bg(app.theme.background)
            .fg(app.theme.surface),
    );
    frame.render_widget(bg, area);

    if app.mode == Mode::LogView {
        status_panel::render_log_view(frame, app, area);
        return;
    }

    let panes = layout::split(area, app.config.display.list_width_pct);

    let (list_area, filter_area) = if app.mode == Mode::Filtering {
        let (fbar, larea) = layout::with_filter_bar(panes.list);
        (larea, Some(fbar))
    } else {
        (panes.list, None)
    };

    unit_list::render(frame, app, list_area, filter_area);

    let (props_area, journal_area) = layout::status_split(panes.status);
    status_panel::render(frame, app, props_area, journal_area);

    if app.mode == Mode::ActionMenu {
        action_menu::render(frame, app);
    }
}
