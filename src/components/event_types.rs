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
