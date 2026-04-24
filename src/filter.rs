use crate::units::UnitEntry;
use nucleo_matcher::{
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
    Config, Matcher,
};

/// Rank `indices` (into `entries`) by fuzzy match against `query`.
/// Returns all indices when query is empty (preserving order).
/// Includes unit_type in the match haystack so `t:service` hints work.
pub fn rank(query: &str, entries: &[UnitEntry], indices: &[usize]) -> Vec<usize> {
    if query.is_empty() {
        return indices.to_vec();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Atom::new(
        query,
        CaseMatching::Smart,
        Normalization::Smart,
        AtomKind::Fuzzy,
        false,
    );

    let mut scored: Vec<(usize, u16)> = indices
        .iter()
        .filter_map(|&i| {
            let e = &entries[i];
            let haystack = format!("{} {} {}", e.name, e.description, e.unit_type);
            let mut buf = Vec::new();
            pattern
                .score(
                    nucleo_matcher::Utf32Str::new(&haystack, &mut buf),
                    &mut matcher,
                )
                .map(|score| (i, score))
        })
        .collect();

    scored.sort_by_key(|&(_, score)| std::cmp::Reverse(score));
    scored.into_iter().map(|(i, _)| i).collect()
}

/// Ordered list of unit types cycled by the `t` key. `None` = all.
pub const TYPE_CYCLE: &[Option<&str>] = &[
    None,
    Some("service"),
    Some("socket"),
    Some("timer"),
    Some("target"),
    Some("mount"),
    Some("path"),
];

/// Return the next type in the cycle after `current`.
pub fn next_type(current: &Option<String>) -> Option<String> {
    let pos = TYPE_CYCLE
        .iter()
        .position(|t| t.map(|s| s.to_string()).as_ref() == current.as_ref())
        .unwrap_or(0);
    let next = TYPE_CYCLE[(pos + 1) % TYPE_CYCLE.len()];
    next.map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::UnitEntry;

    fn make_entry(name: &str, desc: &str, unit_type: &str) -> UnitEntry {
        UnitEntry {
            name: name.to_string(),
            description: desc.to_string(),
            load_state: "loaded".to_string(),
            active_state: "active".to_string(),
            sub_state: "running".to_string(),
            unit_type: unit_type.to_string(),
        }
    }

    #[test]
    fn empty_query_returns_all_indices() {
        let entries = vec![
            make_entry("foo.service", "", "service"),
            make_entry("bar.socket", "", "socket"),
        ];
        let indices = vec![0, 1];
        assert_eq!(rank("", &entries, &indices), vec![0, 1]);
    }

    #[test]
    fn empty_indices_returns_empty() {
        let entries = vec![make_entry("foo.service", "", "service")];
        assert_eq!(rank("foo", &entries, &[]), vec![]);
    }

    #[test]
    fn filters_by_name() {
        let entries = vec![
            make_entry("pipewire.service", "PipeWire", "service"),
            make_entry("bluetooth.service", "Bluetooth", "service"),
            make_entry("networkmanager.service", "Network Manager", "service"),
        ];
        let indices = vec![0, 1, 2];
        let result = rank("pipe", &entries, &indices);
        assert!(result.contains(&0));
        assert!(!result.contains(&1));
    }

    #[test]
    fn type_cycle_wraps_around() {
        // start at None, cycle forward, should eventually return to None
        let mut current: Option<String> = None;
        let start = current.clone();
        for _ in 0..TYPE_CYCLE.len() {
            current = next_type(&current);
        }
        assert_eq!(current, start);
    }

    #[test]
    fn next_type_from_none_is_service() {
        assert_eq!(next_type(&None), Some("service".to_string()));
    }
}
