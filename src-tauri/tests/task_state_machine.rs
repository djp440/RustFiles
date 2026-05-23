use rustfiles::core::tasks::{apply_task_transition, TaskSummary};
use rustfiles::core::types::TaskStatus;

fn sample_summary(status: TaskStatus) -> TaskSummary {
    TaskSummary {
        id: "task-1".to_string(),
        kind: "copy_items".to_string(),
        status,
        message: Some("Ready".to_string()),
        completed_items: vec!["a.txt".to_string()],
        incomplete_items: vec!["b.txt".to_string()],
        unknown_items: vec!["c.txt".to_string()],
        can_cancel: true,
    }
}

#[test]
fn allows_valid_task_transitions() {
    let queued = sample_summary(TaskStatus::Queued);
    let validating = apply_task_transition(&queued, TaskStatus::Validating).expect("queued -> validating");
    assert_eq!(validating.status, TaskStatus::Validating);

    let running = apply_task_transition(&validating, TaskStatus::Running).expect("validating -> running");
    assert_eq!(running.status, TaskStatus::Running);

    let completed = apply_task_transition(&running, TaskStatus::Completed).expect("running -> completed");
    assert_eq!(completed.status, TaskStatus::Completed);
    assert!(!completed.can_cancel, "terminal tasks must not be cancellable");
}

#[test]
fn rejects_invalid_task_transitions() {
    let completed = sample_summary(TaskStatus::Completed);
    assert!(apply_task_transition(&completed, TaskStatus::Running).is_err(), "completed must not return to running");

    let failed = sample_summary(TaskStatus::Failed);
    assert!(apply_task_transition(&failed, TaskStatus::Queued).is_err(), "failed must not return to queued");
}

#[test]
fn preserves_partial_completion_summary() {
    let running = sample_summary(TaskStatus::Running);
    let partial = apply_task_transition(&running, TaskStatus::PartiallyCompleted).expect("running -> partially_completed");

    assert_eq!(partial.status, TaskStatus::PartiallyCompleted);
    assert_eq!(partial.completed_items, vec!["a.txt".to_string()]);
    assert_eq!(partial.incomplete_items, vec!["b.txt".to_string()]);
    assert_eq!(partial.unknown_items, vec!["c.txt".to_string()]);
}
