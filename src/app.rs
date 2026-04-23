use crate::{
    config::Config,
    filter,
    input::{handle_key, AppAction},
    systemd::{do_action, journal_tail, unit_status, Action, Scope},
    theme::Theme,
    units::UnitList,
};
use anyhow::Result;
use crossterm::event::{self, Event};
use ratatui::DefaultTerminal;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Filtering,
    ActionMenu,
    LogView,
}

pub struct App {
    pub config: Config,
    pub theme: Theme,
    pub units: UnitList,
    pub mode: Mode,
    pub filter_query: String,
    pub visible_indices: Vec<usize>,
    pub action_menu_selected: usize,
    pub status_cache: String,
    pub journal_cache: Vec<String>,
    pub log_scroll: usize,
    pub status_message: Option<String>,
}

impl App {
    pub fn new(config: Config, scope: Scope) -> Self {
        let theme = Theme::from_colors(&config.colors);
        let mut units = UnitList::new(scope);
        units.refresh();
        let visible_indices = (0..units.entries.len()).collect();

        Self {
            config,
            theme,
            units,
            mode: Mode::Normal,
            filter_query: String::new(),
            visible_indices,
            action_menu_selected: 0,
            status_cache: String::new(),
            journal_cache: Vec::new(),
            log_scroll: 0,
            status_message: None,
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.refresh_status_cache();
        loop {
            terminal.draw(|frame| crate::ui::draw(frame, &mut self))?;

            let tick = Duration::from_millis(self.config.display.tick_rate_ms);
            if event::poll(tick)? {
                if let Event::Key(key) = event::read()? {
                    let action = handle_key(key, &self.config.keybinds, &self.mode);
                    if self.apply(action) {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn apply(&mut self, action: AppAction) -> bool {
        match action {
            AppAction::Quit => return true,

            AppAction::MoveDown => {
                self.units.move_down(&self.visible_indices);
                self.refresh_status_cache();
            }
            AppAction::MoveUp => {
                self.units.move_up(&self.visible_indices);
                self.refresh_status_cache();
            }
            AppAction::PageDown => {
                self.units.page_down(&self.visible_indices, 10);
                self.refresh_status_cache();
            }
            AppAction::PageUp => {
                self.units.page_up(&self.visible_indices, 10);
                self.refresh_status_cache();
            }
            AppAction::GoTop => {
                self.units.go_top(&self.visible_indices);
                self.refresh_status_cache();
            }
            AppAction::GoBottom => {
                self.units.go_bottom(&self.visible_indices);
                self.refresh_status_cache();
            }

            AppAction::SwitchScope => {
                self.units.switch_scope();
                self.refilter();
                self.refresh_status_cache();
            }

            AppAction::OpenFilter => {
                self.mode = Mode::Filtering;
            }
            AppAction::FilterChar(c) => {
                self.filter_query.push(c);
                self.refilter();
            }
            AppAction::FilterBackspace => {
                self.filter_query.pop();
                self.refilter();
            }
            AppAction::FilterConfirm | AppAction::FilterCancel => {
                if action == AppAction::FilterCancel {
                    self.filter_query.clear();
                    self.refilter();
                }
                self.mode = Mode::Normal;
            }

            AppAction::OpenActionMenu => {
                if self.units.selected_entry().is_some() {
                    self.action_menu_selected = 0;
                    self.mode = Mode::ActionMenu;
                }
            }
            AppAction::ActionUp => {
                let max = Action::all().len();
                if self.action_menu_selected > 0 {
                    self.action_menu_selected -= 1;
                } else {
                    self.action_menu_selected = max - 1;
                }
            }
            AppAction::ActionDown => {
                let max = Action::all().len();
                self.action_menu_selected = (self.action_menu_selected + 1) % max;
            }
            AppAction::ActionConfirm => {
                self.execute_selected_action();
            }
            AppAction::ActionCancel => {
                self.mode = Mode::Normal;
            }

            AppAction::OpenLogs => {
                self.refresh_journal_cache();
                self.log_scroll = 0;
                self.mode = Mode::LogView;
            }
            AppAction::LogScrollDown => {
                if self.log_scroll + 1 < self.journal_cache.len() {
                    self.log_scroll += 3;
                }
            }
            AppAction::LogScrollUp => {
                self.log_scroll = self.log_scroll.saturating_sub(3);
            }
            AppAction::LogClose => {
                self.mode = Mode::Normal;
            }

            AppAction::None => {}
        }
        false
    }

    fn refilter(&mut self) {
        self.visible_indices = filter::rank(&self.filter_query, &self.units.entries);
        if !self.visible_indices.is_empty()
            && !self.visible_indices.contains(&self.units.selected)
        {
            self.units.selected = self.visible_indices[0];
            self.refresh_status_cache();
        }
    }

    fn refresh_status_cache(&mut self) {
        if let Some(entry) = self.units.selected_entry() {
            let name = entry.name.clone();
            let scope = self.units.scope;
            self.status_cache = unit_status(&name, scope).unwrap_or_default();
        }
    }

    fn refresh_journal_cache(&mut self) {
        if let Some(entry) = self.units.selected_entry() {
            let name = entry.name.clone();
            let scope = self.units.scope;
            let lines = self.config.display.journal_lines;
            self.journal_cache = journal_tail(&name, scope, lines).unwrap_or_default();
        }
    }

    fn execute_selected_action(&mut self) {
        let action = &Action::all()[self.action_menu_selected];
        if let Some(entry) = self.units.selected_entry() {
            let name = entry.name.clone();
            let scope = self.units.scope;
            match do_action(action, &name, scope) {
                Ok(_) => {
                    self.status_message =
                        Some(format!("{} {} succeeded", action.label(), name));
                    self.units.refresh();
                    self.refilter();
                    self.refresh_status_cache();
                }
                Err(e) => {
                    self.status_message = Some(format!("Error: {}", e));
                }
            }
        }
        self.mode = Mode::Normal;
    }
}
