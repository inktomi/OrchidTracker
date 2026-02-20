pub struct EventTypeInfo {
    pub key: &'static str,
    pub label: &'static str,
    pub emoji: &'static str,
    pub color_class: &'static str,
    pub bg_class: &'static str,
}

pub const EVENT_TYPES: &[EventTypeInfo] = &[
    EventTypeInfo {
        key: "Flowering",
        label: "Flowering",
        emoji: "\u{1F338}",
        color_class: "text-pink-600 dark:text-pink-400",
        bg_class: "bg-pink-100 dark:bg-pink-900/30",
    },
    EventTypeInfo {
        key: "NewGrowth",
        label: "New Growth",
        emoji: "\u{1F331}",
        color_class: "text-emerald-600 dark:text-emerald-400",
        bg_class: "bg-emerald-100 dark:bg-emerald-900/30",
    },
    EventTypeInfo {
        key: "Repotted",
        label: "Repotted",
        emoji: "\u{1FAB4}",
        color_class: "text-amber-600 dark:text-amber-400",
        bg_class: "bg-amber-100 dark:bg-amber-900/30",
    },
    EventTypeInfo {
        key: "Fertilized",
        label: "Fertilized",
        emoji: "\u{2728}",
        color_class: "text-yellow-600 dark:text-yellow-400",
        bg_class: "bg-yellow-100 dark:bg-yellow-900/30",
    },
    EventTypeInfo {
        key: "PestTreatment",
        label: "Pest Treatment",
        emoji: "\u{1F41B}",
        color_class: "text-red-600 dark:text-red-400",
        bg_class: "bg-red-100 dark:bg-red-900/30",
    },
    EventTypeInfo {
        key: "Purchased",
        label: "Purchased",
        emoji: "\u{1F3F7}\u{FE0F}",
        color_class: "text-violet-600 dark:text-violet-400",
        bg_class: "bg-violet-100 dark:bg-violet-900/30",
    },
    EventTypeInfo {
        key: "Watered",
        label: "Watered",
        emoji: "\u{1F4A7}",
        color_class: "text-sky-600 dark:text-sky-400",
        bg_class: "bg-sky-100 dark:bg-sky-900/30",
    },
    EventTypeInfo {
        key: "Note",
        label: "Note",
        emoji: "\u{1F4DD}",
        color_class: "text-stone-600 dark:text-stone-400",
        bg_class: "bg-stone-100 dark:bg-stone-800",
    },
];

pub fn get_event_info(key: &str) -> Option<&'static EventTypeInfo> {
    EVENT_TYPES.iter().find(|e| e.key == key)
}

/// The allowed event type keys, matching the DB ASSERT constraint in migration 0008.
pub const ALLOWED_EVENT_TYPE_KEYS: &[&str] = &[
    "Flowering", "NewGrowth", "Repotted", "Fertilized",
    "PestTreatment", "Purchased", "Watered", "Note",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_event_types_present() {
        assert_eq!(EVENT_TYPES.len(), 8);
    }

    #[test]
    fn test_event_types_match_allowed_keys() {
        // Ensure every EVENT_TYPES entry has a key in ALLOWED_EVENT_TYPE_KEYS
        for et in EVENT_TYPES {
            assert!(
                ALLOWED_EVENT_TYPE_KEYS.contains(&et.key),
                "EVENT_TYPES key '{}' is not in ALLOWED_EVENT_TYPE_KEYS",
                et.key
            );
        }
        // And vice versa
        for key in ALLOWED_EVENT_TYPE_KEYS {
            assert!(
                EVENT_TYPES.iter().any(|et| et.key == *key),
                "ALLOWED_EVENT_TYPE_KEYS entry '{}' has no EVENT_TYPES entry",
                key
            );
        }
    }

    #[test]
    fn test_no_duplicate_keys() {
        let mut seen = std::collections::HashSet::new();
        for et in EVENT_TYPES {
            assert!(seen.insert(et.key), "Duplicate event type key: {}", et.key);
        }
    }

    #[test]
    fn test_get_event_info_found() {
        let info = get_event_info("Flowering");
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.label, "Flowering");
        assert!(!info.emoji.is_empty());
        assert!(!info.color_class.is_empty());
        assert!(!info.bg_class.is_empty());
    }

    #[test]
    fn test_get_event_info_all_keys() {
        for key in ALLOWED_EVENT_TYPE_KEYS {
            assert!(
                get_event_info(key).is_some(),
                "get_event_info('{}') returned None",
                key
            );
        }
    }

    #[test]
    fn test_get_event_info_not_found() {
        assert!(get_event_info("NonExistent").is_none());
        assert!(get_event_info("").is_none());
    }

    #[test]
    fn test_every_type_has_nonempty_fields() {
        for et in EVENT_TYPES {
            assert!(!et.key.is_empty(), "Event type has empty key");
            assert!(!et.label.is_empty(), "Event type '{}' has empty label", et.key);
            assert!(!et.emoji.is_empty(), "Event type '{}' has empty emoji", et.key);
            assert!(!et.color_class.is_empty(), "Event type '{}' has empty color_class", et.key);
            assert!(!et.bg_class.is_empty(), "Event type '{}' has empty bg_class", et.key);
        }
    }
}
