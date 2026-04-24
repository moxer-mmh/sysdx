use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct Panes {
    pub list: Rect,
    pub status: Rect,
}

pub fn split(area: Rect, list_width_pct: u16) -> Panes {
    let pct = list_width_pct.clamp(20, 70);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(pct),
            Constraint::Percentage(100 - pct),
        ])
        .split(area);

    Panes {
        list: chunks[0],
        status: chunks[1],
    }
}

pub fn with_filter_bar(list_area: Rect) -> (Rect, Rect) {
    // Returns (filter_bar, unit_list)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(list_area);
    (chunks[0], chunks[1])
}

pub fn status_split(status_area: Rect) -> (Rect, Rect) {
    // Returns (properties, journal_preview)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(status_area);
    (chunks[0], chunks[1])
}

/// Carve a 1-row status bar from the bottom of `area`.
/// Returns (main_area, status_bar).
pub fn with_status_bar(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);
    (chunks[0], chunks[1])
}
