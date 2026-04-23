use crate::systemd::{list_units, RawUnit, Scope};

#[derive(Debug, Clone)]
pub struct UnitEntry {
    pub name: String,
    pub description: String,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
    pub unit_type: String,
}

impl UnitEntry {
    pub fn from_raw(raw: RawUnit) -> Self {
        let unit_type = raw
            .unit
            .rsplit_once('.')
            .map(|(_, ext)| ext.to_string())
            .unwrap_or_default();

        Self {
            name: raw.unit,
            description: raw.description,
            load_state: raw.load,
            active_state: raw.active,
            sub_state: raw.sub,
            unit_type,
        }
    }

    pub fn status_indicator(&self) -> &'static str {
        match self.active_state.as_str() {
            "active" => "●",
            "failed" => "✗",
            "inactive" => "○",
            _ => "·",
        }
    }
}

#[derive(Debug)]
pub struct UnitList {
    pub entries: Vec<UnitEntry>,
    pub scope: Scope,
    pub selected: usize,
    pub scroll_offset: usize,
    pub last_error: Option<String>,
}

impl UnitList {
    pub fn new(scope: Scope) -> Self {
        Self {
            entries: Vec::new(),
            scope,
            selected: 0,
            scroll_offset: 0,
            last_error: None,
        }
    }

    pub fn refresh(&mut self) {
        match list_units(self.scope) {
            Ok(raw_units) => {
                self.entries = raw_units.into_iter().map(UnitEntry::from_raw).collect();
                self.last_error = None;
                // clamp selection to valid range
                if self.selected >= self.entries.len() && !self.entries.is_empty() {
                    self.selected = self.entries.len() - 1;
                }
            }
            Err(e) => {
                self.last_error = Some(e.to_string());
            }
        }
    }

    pub fn move_down(&mut self, visible_indices: &[usize]) {
        if visible_indices.is_empty() {
            return;
        }
        let pos = visible_indices.iter().position(|&i| i == self.selected);
        if let Some(p) = pos {
            if p + 1 < visible_indices.len() {
                self.selected = visible_indices[p + 1];
            }
        } else if let Some(&first) = visible_indices.first() {
            self.selected = first;
        }
    }

    pub fn move_up(&mut self, visible_indices: &[usize]) {
        if visible_indices.is_empty() {
            return;
        }
        let pos = visible_indices.iter().position(|&i| i == self.selected);
        if let Some(p) = pos {
            if p > 0 {
                self.selected = visible_indices[p - 1];
            }
        } else if let Some(&first) = visible_indices.first() {
            self.selected = first;
        }
    }

    pub fn go_top(&mut self, visible_indices: &[usize]) {
        if let Some(&first) = visible_indices.first() {
            self.selected = first;
        }
    }

    pub fn go_bottom(&mut self, visible_indices: &[usize]) {
        if let Some(&last) = visible_indices.last() {
            self.selected = last;
        }
    }

    pub fn page_down(&mut self, visible_indices: &[usize], page_size: usize) {
        if visible_indices.is_empty() {
            return;
        }
        let pos = visible_indices
            .iter()
            .position(|&i| i == self.selected)
            .unwrap_or(0);
        let new_pos = (pos + page_size).min(visible_indices.len() - 1);
        self.selected = visible_indices[new_pos];
    }

    pub fn page_up(&mut self, visible_indices: &[usize], page_size: usize) {
        if visible_indices.is_empty() {
            return;
        }
        let pos = visible_indices
            .iter()
            .position(|&i| i == self.selected)
            .unwrap_or(0);
        let new_pos = pos.saturating_sub(page_size);
        self.selected = visible_indices[new_pos];
    }

    pub fn selected_entry(&self) -> Option<&UnitEntry> {
        self.entries.get(self.selected)
    }

    pub fn switch_scope(&mut self) {
        self.scope = match self.scope {
            Scope::User => Scope::System,
            Scope::System => Scope::User,
        };
        self.selected = 0;
        self.scroll_offset = 0;
        self.refresh();
    }
}
