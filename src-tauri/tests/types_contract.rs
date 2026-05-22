use rustfiles::core::types::{TaskStatus, Settings};

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
fn settings_contains_schema_version() {
    let settings = Settings {
        schema_version: 1,
    };
    assert_eq!(settings.schema_version, 1);
    let serialized = serde_json::to_string(&settings).unwrap();
    let deserialized: Settings = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.schema_version, 1);
}
