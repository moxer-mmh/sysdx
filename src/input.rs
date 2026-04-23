use crate::config::KeyBinds;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppAction {
    MoveDown,
    MoveUp,
    PageDown,
    PageUp,
    GoTop,
    GoBottom,
    OpenFilter,
    FilterChar(char),
    FilterBackspace,
    FilterConfirm,
    FilterCancel,
    OpenActionMenu,
    ActionUp,
    ActionDown,
    ActionConfirm,
    ActionCancel,
    SwitchScope,
    OpenLogs,
    LogScrollDown,
    LogScrollUp,
    LogClose,
    Quit,
    None,
}

pub fn handle_key(event: KeyEvent, binds: &KeyBinds, mode: &crate::app::Mode) -> AppAction {
    use crate::app::Mode;

    match mode {
        Mode::Filtering => handle_filter_key(event),
        Mode::ActionMenu => handle_action_menu_key(event),
        Mode::LogView => handle_log_key(event),
        Mode::Normal => handle_normal_key(event, binds),
    }
}

fn handle_normal_key(event: KeyEvent, binds: &KeyBinds) -> AppAction {
    let key_str = key_to_string(event);

    if key_str == binds.quit {
        return AppAction::Quit;
    }
    if key_str == binds.move_down {
        return AppAction::MoveDown;
    }
    if key_str == binds.move_up {
        return AppAction::MoveUp;
    }
    if key_str == binds.page_down {
        return AppAction::PageDown;
    }
    if key_str == binds.page_up {
        return AppAction::PageUp;
    }
    if key_str == binds.go_bottom {
        return AppAction::GoBottom;
    }
    if key_str == binds.go_top {
        return AppAction::GoTop;
    }
    if key_str == binds.filter {
        return AppAction::OpenFilter;
    }
    if key_str == binds.action_menu {
        return AppAction::OpenActionMenu;
    }
    if key_str == binds.switch_scope {
        return AppAction::SwitchScope;
    }
    if key_str == binds.open_logs {
        return AppAction::OpenLogs;
    }

    AppAction::None
}

fn handle_filter_key(event: KeyEvent) -> AppAction {
    match event.code {
        KeyCode::Esc => AppAction::FilterCancel,
        KeyCode::Enter => AppAction::FilterConfirm,
        KeyCode::Backspace => AppAction::FilterBackspace,
        KeyCode::Char(c) => AppAction::FilterChar(c),
        _ => AppAction::None,
    }
}

fn handle_action_menu_key(event: KeyEvent) -> AppAction {
    match event.code {
        KeyCode::Esc | KeyCode::Char('q') => AppAction::ActionCancel,
        KeyCode::Enter => AppAction::ActionConfirm,
        KeyCode::Char('j') | KeyCode::Down => AppAction::ActionDown,
        KeyCode::Char('k') | KeyCode::Up => AppAction::ActionUp,
        _ => AppAction::None,
    }
}

fn handle_log_key(event: KeyEvent) -> AppAction {
    match event.code {
        KeyCode::Esc | KeyCode::Char('q') => AppAction::LogClose,
        KeyCode::Char('j') | KeyCode::Down => AppAction::LogScrollDown,
        KeyCode::Char('k') | KeyCode::Up => AppAction::LogScrollUp,
        KeyCode::Char('d') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            AppAction::LogScrollDown
        }
        KeyCode::Char('u') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            AppAction::LogScrollUp
        }
        _ => AppAction::None,
    }
}

pub fn key_to_string(event: KeyEvent) -> String {
    let ctrl = event.modifiers.contains(KeyModifiers::CONTROL);
    match event.code {
        KeyCode::Char(c) if ctrl => format!("ctrl-{}", c),
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "enter".to_string(),
        KeyCode::Tab => "tab".to_string(),
        KeyCode::Esc => "esc".to_string(),
        KeyCode::Up => "up".to_string(),
        KeyCode::Down => "down".to_string(),
        KeyCode::Left => "left".to_string(),
        KeyCode::Right => "right".to_string(),
        _ => String::new(),
    }
}
