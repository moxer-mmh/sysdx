use crate::{
    config::Config,
    filter,
    input::{handle_key, AppAction},
    systemd::{
        do_action, journal_tail, list_units, unit_file, unit_status, Action, RawUnit, Scope,
    },
    theme::Theme,
    units::UnitList,
};
use anyhow::Result;
use crossterm::event::Event;
use futures::StreamExt;
use ratatui::DefaultTerminal;
use std::time::Duration;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Filtering,
    ActionMenu,
    LogView,
    Confirm,
    UnitFile,
    Help,
}

/// Messages sent from background tasks back to the main loop.
pub enum BgMsg {
    Units(Vec<RawUnit>),
    Status(String),
    Journal(Vec<String>),
    UnitFile(Vec<String>),
    ActionDone { label: String, unit: String },
    ActionError(String),
}

pub struct App {
    pub config: Config,
    pub theme: Theme,
    pub units: UnitList,
    pub mode: Mode,
    pub filter_query: String,
    pub type_filter: Option<String>,
    pub visible_indices: Vec<usize>,
    pub action_menu_selected: usize,
    pub confirm_pending: Option<usize>,
    pub status_cache: String,
    pub journal_cache: Vec<String>,
    pub log_scroll: usize,
    pub log_live: bool,
    pub unit_file_cache: Vec<String>,
    pub unit_file_scroll: usize,
    pub status_message: Option<String>,
    pub status_message_ticks: u8,
    pub loading: bool,
    refresh_ticks: u64,
    tx: UnboundedSender<BgMsg>,
    rx: UnboundedReceiver<BgMsg>,
}

impl App {
    pub fn new(config: Config, scope: Scope) -> Self {
        let theme = Theme::from_colors(&config.colors);
        let mut units = UnitList::new(scope);
        units.refresh(); // synchronous initial load
        let visible_indices = (0..units.entries.len()).collect();
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            config,
            theme,
            units,
            mode: Mode::Normal,
            filter_query: String::new(),
            type_filter: None,
            visible_indices,
            action_menu_selected: 0,
            confirm_pending: None,
            status_cache: String::new(),
            journal_cache: Vec::new(),
            log_scroll: 0,
            log_live: false,
            unit_file_cache: Vec::new(),
            unit_file_scroll: 0,
            status_message: None,
            status_message_ticks: 0,
            loading: false,
            refresh_ticks: 0,
            tx,
            rx,
        }
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.spawn_refresh_status();

        let mut events = crossterm::event::EventStream::new();
        let tick_dur = Duration::from_millis(self.config.display.tick_rate_ms);
        let mut tick_interval = tokio::time::interval(tick_dur);
        tick_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            terminal.draw(|frame| crate::ui::draw(frame, &mut self))?;

            tokio::select! {
                ev = events.next() => {
                    if let Some(Ok(Event::Key(key))) = ev {
                        let action = handle_key(key, &self.config.keybinds, &self.mode);
                        if self.apply(action) {
                            break;
                        }
                    }
                }
                _ = tick_interval.tick() => {
                    self.on_tick();
                }
                msg = self.rx.recv() => {
                    if let Some(msg) = msg {
                        self.handle_bg(msg);
                    }
                }
            }
        }
        Ok(())
    }

    fn on_tick(&mut self) {
        // Auto-refresh
        if self.config.display.auto_refresh_secs > 0 {
            let ticks_per_refresh = (self.config.display.auto_refresh_secs * 1000)
                / self.config.display.tick_rate_ms.max(1);
            self.refresh_ticks += 1;
            if self.refresh_ticks >= ticks_per_refresh {
                self.refresh_ticks = 0;
                self.spawn_refresh_units();
            }
        }

        // Live-tail logs
        if self.mode == Mode::LogView && self.log_live {
            self.spawn_refresh_journal();
        }

        // Status message timeout (~3 s at default 250 ms tick)
        if self.status_message.is_some() {
            if self.status_message_ticks > 0 {
                self.status_message_ticks -= 1;
            } else {
                self.status_message = None;
            }
        }
    }

    fn handle_bg(&mut self, msg: BgMsg) {
        match msg {
            BgMsg::Units(raw) => {
                self.units.apply_units(raw);
                self.loading = false;
                self.refilter();
            }
            BgMsg::Status(s) => {
                self.status_cache = s;
            }
            BgMsg::Journal(lines) => {
                self.journal_cache = lines;
            }
            BgMsg::UnitFile(lines) => {
                self.unit_file_cache = lines;
            }
            BgMsg::ActionDone { label, unit } => {
                self.set_message(format!("{} {} succeeded", label, unit));
                self.spawn_refresh_units();
                self.spawn_refresh_status();
                self.mode = Mode::Normal;
            }
            BgMsg::ActionError(e) => {
                self.set_message(format!("Error: {}", e));
                self.mode = Mode::Normal;
            }
        }
    }

    fn apply(&mut self, action: AppAction) -> bool {
        match action {
            AppAction::Quit => return true,

            AppAction::MoveDown => {
                self.units.move_down(&self.visible_indices);
                self.spawn_refresh_status();
            }
            AppAction::MoveUp => {
                self.units.move_up(&self.visible_indices);
                self.spawn_refresh_status();
            }
            AppAction::PageDown => {
                self.units.page_down(&self.visible_indices, 10);
                self.spawn_refresh_status();
            }
            AppAction::PageUp => {
                self.units.page_up(&self.visible_indices, 10);
                self.spawn_refresh_status();
            }
            AppAction::GoTop => {
                self.units.go_top(&self.visible_indices);
                self.spawn_refresh_status();
            }
            AppAction::GoBottom => {
                self.units.go_bottom(&self.visible_indices);
                self.spawn_refresh_status();
            }

            AppAction::SwitchScope => {
                self.units.switch_scope();
                self.refilter();
                self.loading = true;
                self.spawn_refresh_units();
                self.spawn_refresh_status();
            }

            AppAction::Refresh => {
                self.loading = true;
                self.spawn_refresh_units();
                self.spawn_refresh_status();
            }

            AppAction::FilterType => {
                self.type_filter = filter::next_type(&self.type_filter);
                self.refilter();
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
                self.action_menu_selected = if self.action_menu_selected > 0 {
                    self.action_menu_selected - 1
                } else {
                    max - 1
                };
            }
            AppAction::ActionDown => {
                self.action_menu_selected = (self.action_menu_selected + 1) % Action::all().len();
            }
            AppAction::ActionConfirm => {
                self.maybe_confirm_or_execute();
            }
            AppAction::ActionCancel => {
                self.mode = Mode::Normal;
            }

            AppAction::ConfirmYes => {
                if let Some(idx) = self.confirm_pending.take() {
                    self.execute_action_at(idx);
                }
            }
            AppAction::ConfirmNo => {
                self.confirm_pending = None;
                self.mode = Mode::Normal;
            }

            AppAction::OpenLogs => {
                self.log_scroll = 0;
                self.log_live = false;
                self.spawn_refresh_journal();
                self.mode = Mode::LogView;
            }
            AppAction::LogScrollDown => {
                self.log_scroll = self
                    .log_scroll
                    .saturating_add(3)
                    .min(self.journal_cache.len().saturating_sub(1));
            }
            AppAction::LogScrollUp => {
                self.log_scroll = self.log_scroll.saturating_sub(3);
            }
            AppAction::LogToggleLiveTail => {
                self.log_live = !self.log_live;
                if self.log_live {
                    self.spawn_refresh_journal();
                }
            }
            AppAction::LogClose => {
                self.log_live = false;
                self.mode = Mode::Normal;
            }

            AppAction::OpenUnitFile => {
                if self.units.selected_entry().is_some() {
                    self.unit_file_scroll = 0;
                    self.spawn_refresh_unit_file();
                    self.mode = Mode::UnitFile;
                }
            }
            AppAction::UnitFileScrollDown => {
                self.unit_file_scroll = self
                    .unit_file_scroll
                    .saturating_add(3)
                    .min(self.unit_file_cache.len().saturating_sub(1));
            }
            AppAction::UnitFileScrollUp => {
                self.unit_file_scroll = self.unit_file_scroll.saturating_sub(3);
            }
            AppAction::UnitFileClose => {
                self.mode = Mode::Normal;
            }

            AppAction::OpenHelp => {
                self.mode = Mode::Help;
            }
            AppAction::HelpClose => {
                self.mode = Mode::Normal;
            }

            AppAction::None => {}
        }
        false
    }

    fn maybe_confirm_or_execute(&mut self) {
        let action = &Action::all()[self.action_menu_selected];
        if self.config.display.confirm_destructive && action.is_destructive() {
            self.confirm_pending = Some(self.action_menu_selected);
            self.mode = Mode::Confirm;
        } else {
            self.execute_action_at(self.action_menu_selected);
        }
    }

    fn execute_action_at(&mut self, action_idx: usize) {
        let action = Action::all()[action_idx].clone();
        if let Some(entry) = self.units.selected_entry() {
            let name = entry.name.clone();
            let scope = self.units.scope;
            let label = action.label().to_string();
            let tx = self.tx.clone();
            tokio::task::spawn_blocking(move || match do_action(&action, &name, scope) {
                Ok(_) => {
                    let _ = tx.send(BgMsg::ActionDone { label, unit: name });
                }
                Err(e) => {
                    let _ = tx.send(BgMsg::ActionError(e.to_string()));
                }
            });
        }
        self.mode = Mode::Normal;
    }

    fn refilter(&mut self) {
        let type_filtered: Vec<usize> = match &self.type_filter {
            Some(t) => self
                .units
                .entries
                .iter()
                .enumerate()
                .filter(|(_, e)| &e.unit_type == t)
                .map(|(i, _)| i)
                .collect(),
            None => (0..self.units.entries.len()).collect(),
        };

        self.visible_indices =
            filter::rank(&self.filter_query, &self.units.entries, &type_filtered);

        if !self.visible_indices.is_empty() && !self.visible_indices.contains(&self.units.selected)
        {
            self.units.selected = self.visible_indices[0];
            self.spawn_refresh_status();
        }
    }

    fn spawn_refresh_units(&self) {
        let tx = self.tx.clone();
        let scope = self.units.scope;
        tokio::task::spawn_blocking(move || {
            if let Ok(units) = list_units(scope) {
                let _ = tx.send(BgMsg::Units(units));
            }
        });
    }

    fn spawn_refresh_status(&self) {
        if let Some(entry) = self.units.selected_entry() {
            let name = entry.name.clone();
            let scope = self.units.scope;
            let tx = self.tx.clone();
            tokio::task::spawn_blocking(move || {
                if let Ok(s) = unit_status(&name, scope) {
                    let _ = tx.send(BgMsg::Status(s));
                }
            });
        }
    }

    fn spawn_refresh_journal(&self) {
        if let Some(entry) = self.units.selected_entry() {
            let name = entry.name.clone();
            let scope = self.units.scope;
            let lines = self.config.display.journal_lines;
            let tx = self.tx.clone();
            tokio::task::spawn_blocking(move || {
                if let Ok(j) = journal_tail(&name, scope, lines) {
                    let _ = tx.send(BgMsg::Journal(j));
                }
            });
        }
    }

    fn spawn_refresh_unit_file(&self) {
        if let Some(entry) = self.units.selected_entry() {
            let name = entry.name.clone();
            let scope = self.units.scope;
            let tx = self.tx.clone();
            tokio::task::spawn_blocking(move || {
                if let Ok(lines) = unit_file(&name, scope) {
                    let _ = tx.send(BgMsg::UnitFile(lines));
                }
            });
        }
    }

    fn set_message(&mut self, msg: String) {
        self.status_message = Some(msg);
        self.status_message_ticks = 12; // ~3 s at 250 ms tick
    }

    pub fn status_bar_text(&self) -> String {
        // Error from unit list takes top priority
        if let Some(err) = &self.units.last_error {
            return format!(" ✗ {}", err);
        }
        // Then a transient status message (action result / error)
        if let Some(msg) = &self.status_message {
            return format!(" {}", msg);
        }
        // Otherwise show mode-sensitive hints
        let type_label = self
            .type_filter
            .as_deref()
            .map(|t| format!(" [type:{}]", t))
            .unwrap_or_default();
        let loading = if self.loading { " ⟳" } else { "" };

        match self.mode {
            Mode::Normal => format!(
                "{}{}  j/k navigate  g/G top/bot  / filter  t type  enter action  l logs  u unit-file  r refresh  ? help  q quit",
                type_label, loading
            ),
            Mode::Filtering => "  type to search  enter confirm  esc cancel".to_string(),
            Mode::ActionMenu => "  j/k navigate  enter confirm  esc cancel".to_string(),
            Mode::LogView => {
                let live = if self.log_live { " [LIVE]" } else { "" };
                format!("  j/k scroll  f toggle live-tail{}  esc/q close", live)
            }
            Mode::Confirm => "  y confirm  n/esc cancel".to_string(),
            Mode::UnitFile => "  j/k scroll  ctrl-d/u page  esc/q close".to_string(),
            Mode::Help => "  esc/q close".to_string(),
        }
    }
}
