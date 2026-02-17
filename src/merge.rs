use std::collections::HashMap;

use crate::orchid::{LogEntry, Orchid};

/// Merge local and remote orchid lists by union on orchid ID.
/// Local metadata wins for shared IDs; history entries are unioned by entry ID.
/// Remote-only orchids are appended. Result is sorted by ID for determinism.
pub fn merge_orchids(local: Vec<Orchid>, remote: Vec<Orchid>) -> Vec<Orchid> {
    let mut remote_map: HashMap<u64, Orchid> = remote.into_iter().map(|o| (o.id, o)).collect();

    let mut merged: Vec<Orchid> = local
        .into_iter()
        .map(|mut local_orchid| {
            if let Some(remote_orchid) = remote_map.remove(&local_orchid.id) {
                local_orchid.history =
                    merge_history(local_orchid.history, remote_orchid.history);
            }
            local_orchid
        })
        .collect();

    // Append remote-only orchids
    let mut remaining: Vec<Orchid> = remote_map.into_values().collect();
    remaining.sort_by_key(|o| o.id);
    merged.extend(remaining);

    merged.sort_by_key(|o| o.id);
    merged
}

/// Merge history entries by union on entry ID. Local wins on collision.
/// Result is sorted by timestamp.
fn merge_history(local: Vec<LogEntry>, remote: Vec<LogEntry>) -> Vec<LogEntry> {
    let mut map: HashMap<u64, LogEntry> = remote.into_iter().map(|e| (e.id, e)).collect();

    for entry in local {
        map.insert(entry.id, entry);
    }

    let mut entries: Vec<LogEntry> = map.into_values().collect();
    entries.sort_by_key(|e| e.timestamp);
    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchid::{LightRequirement, Placement};
    use chrono::{TimeZone, Utc};

    fn make_orchid(id: u64, name: &str) -> Orchid {
        Orchid {
            id,
            name: name.into(),
            species: "Test Species".into(),
            water_frequency_days: 7,
            light_requirement: LightRequirement::Medium,
            notes: String::new(),
            placement: Placement::Medium,
            light_lux: String::new(),
            temperature_range: String::new(),
            conservation_status: None,
            history: Vec::new(),
        }
    }

    fn make_entry(id: u64, note: &str, hour: u32) -> LogEntry {
        LogEntry {
            id,
            timestamp: Utc.with_ymd_and_hms(2025, 1, 1, hour, 0, 0).unwrap(),
            note: note.into(),
            image_data: None,
        }
    }

    #[test]
    fn test_both_empty() {
        let result = merge_orchids(vec![], vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_local_only() {
        let local = vec![make_orchid(1, "Local")];
        let result = merge_orchids(local, vec![]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Local");
    }

    #[test]
    fn test_remote_only() {
        let remote = vec![make_orchid(2, "Remote")];
        let result = merge_orchids(vec![], remote);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Remote");
    }

    #[test]
    fn test_disjoint_sets() {
        let local = vec![make_orchid(1, "A")];
        let remote = vec![make_orchid(2, "B")];
        let result = merge_orchids(local, remote);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[1].id, 2);
    }

    #[test]
    fn test_same_id_local_metadata_wins() {
        let local = vec![make_orchid(1, "Local Name")];
        let remote = vec![make_orchid(1, "Remote Name")];
        let result = merge_orchids(local, remote);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Local Name");
    }

    #[test]
    fn test_history_union() {
        let mut local_orchid = make_orchid(1, "Test");
        local_orchid.history = vec![make_entry(10, "local entry", 1)];

        let mut remote_orchid = make_orchid(1, "Test Remote");
        remote_orchid.history = vec![make_entry(20, "remote entry", 2)];

        let result = merge_orchids(vec![local_orchid], vec![remote_orchid]);
        assert_eq!(result[0].history.len(), 2);
        assert_eq!(result[0].history[0].note, "local entry");
        assert_eq!(result[0].history[1].note, "remote entry");
    }

    #[test]
    fn test_history_dedup_local_wins() {
        let mut local_orchid = make_orchid(1, "Test");
        local_orchid.history = vec![make_entry(10, "local version", 1)];

        let mut remote_orchid = make_orchid(1, "Test");
        remote_orchid.history = vec![make_entry(10, "remote version", 1)];

        let result = merge_orchids(vec![local_orchid], vec![remote_orchid]);
        assert_eq!(result[0].history.len(), 1);
        assert_eq!(result[0].history[0].note, "local version");
    }

    #[test]
    fn test_sort_order() {
        let local = vec![make_orchid(3, "C"), make_orchid(1, "A")];
        let remote = vec![make_orchid(2, "B")];
        let result = merge_orchids(local, remote);
        let ids: Vec<u64> = result.iter().map(|o| o.id).collect();
        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn test_complex_multi_orchid() {
        let mut local1 = make_orchid(1, "Shared Local");
        local1.history = vec![make_entry(100, "L1", 1), make_entry(101, "L2", 3)];
        let local2 = make_orchid(3, "Local Only");

        let mut remote1 = make_orchid(1, "Shared Remote");
        remote1.history = vec![make_entry(100, "R1", 1), make_entry(102, "R2", 2)];
        let remote2 = make_orchid(2, "Remote Only");

        let result = merge_orchids(vec![local1, local2], vec![remote1, remote2]);

        assert_eq!(result.len(), 3);

        // ID 1: local metadata wins
        assert_eq!(result[0].name, "Shared Local");
        // History: 100 (local wins), 101 (local only), 102 (remote only) = 3 entries
        assert_eq!(result[0].history.len(), 3);
        assert_eq!(result[0].history[0].note, "L1"); // id 100, local wins
        assert_eq!(result[0].history[1].note, "R2"); // id 102, hour 2
        assert_eq!(result[0].history[2].note, "L2"); // id 101, hour 3

        // ID 2: remote only
        assert_eq!(result[1].name, "Remote Only");

        // ID 3: local only
        assert_eq!(result[2].name, "Local Only");
    }
}
