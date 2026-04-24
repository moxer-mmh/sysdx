use crate::config::KeyBinds;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppAction {
    // Navigation
    MoveDown,
    MoveUp,
    PageDown,
    PageUp,
    GoTop,
    GoBottom,
    // Filter
    OpenFilter,
    FilterChar(char),
    FilterBackspace,
    FilterConfirm,
    FilterCancel,
    // Type cycling
    FilterType,
    // Action menu
    OpenActionMenu,
    ActionUp,
    ActionDown,
    ActionConfirm,
    ActionCancel,
    // Confirmation dialog
    ConfirmYes,
    ConfirmNo,
    // Scope
    SwitchScope,
    // Log view
    OpenLogs,
    LogScrollDown,
    LogScrollUp,
    LogClose,
    LogToggleLiveTail,
    // Unit file view
    OpenUnitFile,
    UnitFileScrollDown,
    UnitFileScrollUp,
    UnitFileClose,
    // Help
    OpenHelp,
    HelpClose,
    // Misc
    Refresh,
    Quit,
    None,
}

pub fn handle_key(event: KeyEvent, binds: &KeyBinds, mode: &crate::app::Mode) -> AppAction {
    use crate::app::Mode;

    match mode {
        Mode::Filtering => handle_filter_key(event),
        Mode::ActionMenu => handle_action_menu_key(event),
        Mode::LogView => handle_log_key(event),
        Mode::Confirm => handle_confirm_key(event),
        Mode::UnitFile => handle_unit_file_key(event),
        Mode::Help => handle_help_key(event),
        Mode::Normal => handle_normal_key(event, binds),
    }
}

fn handle_normal_key(event: KeyEvent, binds: &KeyBinds) -> AppAction {
    let key_str = key_to_string(event);

    if key_str == binds.quit { return AppAction::Quit; }
    if key_str == binds.move_down { return AppAction::MoveDown; }
    if key_str == binds.move_up { return AppAction::MoveUp; }
    if key_str == binds.page_down { return AppAction::PageDown; }
    if key_str == binds.page_up { return AppAction::PageUp; }
    if key_str == binds.go_bottom { return AppAction::GoBottom; }
    if key_str == binds.go_top { return AppAction::GoTop; }
    if key_str == binds.filter { return AppAction::OpenFilter; }
    if key_str == binds.action_menu { return AppAction::OpenActionMenu; }
    if key_str == binds.switch_scope { return AppAction::SwitchScope; }
    if key_str == binds.open_logs { return AppAction::OpenLogs; }
    if key_str == binds.refresh { return AppAction::Refresh; }
    if key_str == binds.open_unit_file { return AppAction::OpenUnitFile; }
    if key_str == binds.help { return AppAction::OpenHelp; }
    if key_str == binds.type_filter { return AppAction::FilterType; }

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
        KeyCode::Char('f') => AppAction::LogToggleLiveTail,
        KeyCode::Char('d') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            AppAction::LogScrollDown
        }
        KeyCode::Char('u') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            AppAction::LogScrollUp
        }
        _ => AppAction::None,
    }
}

fn handle_confirm_key(event: KeyEvent) -> AppAction {
    match event.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => AppAction::ConfirmYes,
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Char('q') => {
            AppAction::ConfirmNo
        }
        _ => AppAction::None,
    }
}

fn handle_unit_file_key(event: KeyEvent) -> AppAction {
    match event.code {
        KeyCode::Esc | KeyCode::Char('q') => AppAction::UnitFileClose,
        KeyCode::Char('j') | KeyCode::Down => AppAction::UnitFileScrollDown,
        KeyCode::Char('k') | KeyCode::Up => AppAction::UnitFileScrollUp,
        KeyCode::Char('d') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            AppAction::UnitFileScrollDown
        }
        KeyCode::Char('u') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            AppAction::UnitFileScrollUp
        }
        _ => AppAction::None,
    }
}

fn handle_help_key(event: KeyEvent) -> AppAction {
    match event.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => AppAction::HelpClose,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl_key(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    #[test]
    fn log_f_toggles_live_tail() {
        let action = handle_log_key(key(KeyCode::Char('f')));
        assert_eq!(action, AppAction::LogToggleLiveTail);
    }

    #[test]
    fn confirm_y_accepts() {
        assert_eq!(handle_confirm_key(key(KeyCode::Char('y'))), AppAction::ConfirmYes);
        assert_eq!(handle_confirm_key(key(KeyCode::Char('Y'))), AppAction::ConfirmYes);
    }

    #[test]
    fn confirm_esc_cancels() {
        assert_eq!(handle_confirm_key(key(KeyCode::Esc)), AppAction::ConfirmNo);
        assert_eq!(handle_confirm_key(key(KeyCode::Char('n'))), AppAction::ConfirmNo);
    }

    #[test]
    fn ctrl_d_scrolls_log_down() {
        let action = handle_log_key(ctrl_key('d'));
        assert_eq!(action, AppAction::LogScrollDown);
    }

    #[test]
    fn normal_key_to_string_roundtrip() {
        assert_eq!(key_to_string(key(KeyCode::Char('j'))), "j");
        assert_eq!(key_to_string(key(KeyCode::Enter)), "enter");
        assert_eq!(key_to_string(key(KeyCode::Tab)), "tab");
        assert_eq!(key_to_string(ctrl_key('d')), "ctrl-d");
    }
}
