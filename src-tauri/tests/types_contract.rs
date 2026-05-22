use rustfiles::core::types::{Settings, SortKey, TaskStatus};

#[test]
fn task_status_contains_all_required_states() {
    let variants: Vec<String> = vec![
        serde_json::to_string(&TaskStatus::Queued).unwrap(),
        serde_json::to_string(&TaskStatus::Validating).unwrap(),
        serde_json::to_string(&TaskStatus::Running).unwrap(),
        serde_json::to_string(&TaskStatus::WaitingForConflictDecision).unwrap(),
        serde_json::to_string(&TaskStatus::Cancelling).unwrap(),
        serde_json::to_string(&TaskStatus::Cancelled).unwrap(),
        serde_json::to_string(&TaskStatus::Completed).unwrap(),
        serde_json::to_string(&TaskStatus::Failed).unwrap(),
        serde_json::to_string(&TaskStatus::PartiallyCompleted).unwrap(),
    ];

    assert_eq!(variants.len(), 9);

    let expected = vec![
        "\"queued\"",
        "\"validating\"",
        "\"running\"",
        "\"waiting_for_conflict_decision\"",
        "\"cancelling\"",
        "\"cancelled\"",
        "\"completed\"",
        "\"failed\"",
        "\"partially_completed\"",
    ];

    for (serialized, expected_str) in variants.iter().zip(expected.iter()) {
        assert_eq!(serialized, expected_str, "expected {} but got {}", expected_str, serialized);
    }
}

#[test]
fn settings_contains_required_fields() {
    let settings = Settings {
        schema_version: 1,
        show_hidden_files: false,
        show_file_extensions: true,
        sort_key: SortKey::Name,
        sort_ascending: true,
    };
    assert_eq!(settings.schema_version, 1);
    assert!(!settings.show_hidden_files);
    assert!(settings.show_file_extensions);
    assert_eq!(settings.sort_key, SortKey::Name);
    assert!(settings.sort_ascending);
}

#[test]
fn settings_serialize_roundtrip() {
    let settings = Settings {
        schema_version: 1,
        show_hidden_files: true,
        show_file_extensions: false,
        sort_key: SortKey::Size,
        sort_ascending: false,
    };
    let serialized = serde_json::to_string(&settings).unwrap();
    let deserialized: Settings = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.schema_version, 1);
    assert!(deserialized.show_hidden_files);
    assert!(!deserialized.show_file_extensions);
    assert_eq!(deserialized.sort_key, SortKey::Size);
    assert!(!deserialized.sort_ascending);
}

#[test]
fn settings_serialized_keys_use_snake_case() {
    let settings = Settings {
        schema_version: 1,
        show_hidden_files: false,
        show_file_extensions: true,
        sort_key: SortKey::Name,
        sort_ascending: true,
    };
    let serialized = serde_json::to_string(&settings).unwrap();
    assert!(serialized.contains("show_hidden_files"), "expected snake_case key");
    assert!(serialized.contains("show_file_extensions"), "expected snake_case key");
    assert!(serialized.contains("sort_key"), "expected snake_case key");
    assert!(serialized.contains("sort_ascending"), "expected snake_case key");
}
