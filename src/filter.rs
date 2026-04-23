use crate::units::UnitEntry;
use nucleo_matcher::{
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
    Config, Matcher,
};

pub fn rank(query: &str, entries: &[UnitEntry]) -> Vec<usize> {
    if query.is_empty() {
        return (0..entries.len()).collect();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Atom::new(
        query,
        CaseMatching::Smart,
        Normalization::Smart,
        AtomKind::Fuzzy,
        false,
    );

    let mut scored: Vec<(usize, u16)> = entries
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            let haystack = format!("{} {}", e.name, e.description);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{systemd::Scope, units::UnitEntry};

    fn make_entry(name: &str, desc: &str) -> UnitEntry {
        UnitEntry {
            name: name.to_string(),
            description: desc.to_string(),
            load_state: "loaded".to_string(),
            active_state: "active".to_string(),
            sub_state: "running".to_string(),
            unit_type: "service".to_string(),
        }
    }

    #[test]
    fn empty_query_returns_all() {
        let entries = vec![make_entry("foo.service", ""), make_entry("bar.service", "")];
        let result = rank("", &entries);
        assert_eq!(result, vec![0, 1]);
    }

    #[test]
    fn filters_by_name() {
        let entries = vec![
            make_entry("pipewire.service", "PipeWire"),
            make_entry("bluetooth.service", "Bluetooth"),
            make_entry("networkmanager.service", "Network"),
        ];
        let result = rank("pipe", &entries);
        assert!(result.contains(&0));
        assert!(!result.contains(&1));
    }
}
