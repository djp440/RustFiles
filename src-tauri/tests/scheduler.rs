use std::future::Future;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use rustfiles::commands;
use rustfiles::core::observability;
use rustfiles::core::scheduler::{
    self, PriorityKind, SchedulerReportKind, SchedulerSignal, VisibleRange, WorkKind,
};
use std::sync::{Mutex, OnceLock};

fn make_signal(
    active_tab_id: &str,
    visible_range: Option<VisibleRange>,
    last_input_at_ms: Option<u64>,
    is_scrolling: bool,
    interaction_epoch: u64,
) -> SchedulerSignal {
    SchedulerSignal {
        active_tab_id: active_tab_id.to_string(),
        visible_range,
        last_input_at_ms,
        is_scrolling,
        interaction_epoch,
    }
}

fn noop_raw_waker() -> RawWaker {
    fn clone(_: *const ()) -> RawWaker {
        noop_raw_waker()
    }

    fn wake(_: *const ()) {}
    fn wake_by_ref(_: *const ()) {}
    fn drop(_: *const ()) {}

    RawWaker::new(
        std::ptr::null(),
        &RawWakerVTable::new(clone, wake, wake_by_ref, drop),
    )
}

fn block_on_ready<F: Future>(future: F) -> F::Output {
    let waker = unsafe { Waker::from_raw(noop_raw_waker()) };
    let mut context = Context::from_waker(&waker);
    let mut future = Box::pin(future);

    match future.as_mut().poll(&mut context) {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("future should resolve immediately"),
    }
}

fn priority_score_for(
    signal: &SchedulerSignal,
    work_kind: WorkKind,
) -> (PriorityKind, u16) {
    scheduler::priority_snapshot(signal)
        .into_iter()
        .find(|item| item.work_kind == work_kind)
        .map(|item| (item.priority_kind, item.score))
        .expect("expected work kind to be present in snapshot")
}

fn scheduler_test_guard() -> std::sync::MutexGuard<'static, ()> {
    static TEST_GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_GUARD
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("scheduler test guard lock poisoned")
}

#[test]
fn scheduler_default_priority_order_matches_architecture_plan() {
    let _guard = scheduler_test_guard();
    let signal = make_signal(
        "tab-1",
        Some(VisibleRange {
            start_index: 0,
            end_index: 63,
        }),
        Some(1_000),
        false,
        7,
    );

    let order = scheduler::priority_order(&signal);
    assert_eq!(
        order,
        vec![
            WorkKind::ForegroundDirectoryEnumeration,
            WorkKind::VisibleThumbnail,
            WorkKind::ForegroundSearch,
            WorkKind::FileProgress,
            WorkKind::BackgroundRefresh,
            WorkKind::NonVisibleThumbnail,
            WorkKind::BackgroundRecursiveSearch,
        ]
    );

    let json = serde_json::to_value(&signal).expect("signal should serialize");
    assert_eq!(json["active_tab_id"], "tab-1");
    assert_eq!(json["visible_range"]["start_index"], 0);
    assert_eq!(json["visible_range"]["end_index"], 63);
    assert_eq!(json["last_input_at_ms"], 1_000);
    assert_eq!(json["is_scrolling"], false);
    assert_eq!(json["interaction_epoch"], 7);
}

#[test]
fn scheduler_scrolling_lowers_non_visible_work_priority() {
    let _guard = scheduler_test_guard();
    let steady = make_signal(
        "tab-1",
        Some(VisibleRange {
            start_index: 0,
            end_index: 63,
        }),
        Some(1_000),
        false,
        9,
    );
    let scrolling = make_signal(
        "tab-1",
        Some(VisibleRange {
            start_index: 0,
            end_index: 63,
        }),
        Some(1_000),
        true,
        9,
    );

    let (_, steady_non_visible_thumbnail_score) =
        priority_score_for(&steady, WorkKind::NonVisibleThumbnail);
    let (_, scrolling_non_visible_thumbnail_score) =
        priority_score_for(&scrolling, WorkKind::NonVisibleThumbnail);
    assert!(
        scrolling_non_visible_thumbnail_score > steady_non_visible_thumbnail_score,
        "scrolling should lower non-visible thumbnail priority"
    );

    let (_, steady_recursive_search_score) =
        priority_score_for(&steady, WorkKind::BackgroundRecursiveSearch);
    let (_, scrolling_recursive_search_score) =
        priority_score_for(&scrolling, WorkKind::BackgroundRecursiveSearch);
    assert!(
        scrolling_recursive_search_score > steady_recursive_search_score,
        "scrolling should lower recursive background search priority"
    );
}

#[test]
fn scheduler_interaction_epoch_staleness_is_detectable() {
    let _guard = scheduler_test_guard();
    assert!(scheduler::is_interaction_epoch_stale(9, 8));
    assert!(!scheduler::is_interaction_epoch_stale(8, 8));
    assert!(!scheduler::is_interaction_epoch_stale(8, 9));
}

#[test]
fn scheduler_viewport_and_interaction_commands_return_structured_ack_and_samples() {
    let _guard = scheduler_test_guard();
    scheduler::reset_scheduler_state();
    observability::clear_performance_samples();

    let viewport_signal = make_signal(
        "tab-2",
        Some(VisibleRange {
            start_index: 0,
            end_index: 31,
        }),
        Some(2_000),
        true,
        11,
    );
    let viewport_ack = block_on_ready(commands::report_viewport_state(viewport_signal))
        .expect("viewport report should succeed");
    assert_eq!(viewport_ack.report_kind, SchedulerReportKind::Viewport);
    assert!(viewport_ack.accepted);
    assert_eq!(viewport_ack.active_tab_id, "tab-2");
    assert_eq!(viewport_ack.interaction_epoch, 11);
    assert!(viewport_ack.frame_budget_degraded);
    assert_eq!(
        viewport_ack.priority_order,
        vec![
            WorkKind::ForegroundDirectoryEnumeration,
            WorkKind::VisibleThumbnail,
            WorkKind::ForegroundSearch,
            WorkKind::FileProgress,
            WorkKind::BackgroundRefresh,
            WorkKind::NonVisibleThumbnail,
            WorkKind::BackgroundRecursiveSearch,
        ]
    );
    assert!(viewport_ack.summary.contains("viewport ack"));

    let interaction_signal = make_signal(
        "tab-2",
        Some(VisibleRange {
            start_index: 0,
            end_index: 31,
        }),
        Some(observability::now_ms().saturating_sub(8)),
        false,
        12,
    );
    let interaction_ack = block_on_ready(commands::report_interaction_state(interaction_signal))
        .expect("interaction report should succeed");
    assert_eq!(interaction_ack.report_kind, SchedulerReportKind::Interaction);
    assert!(interaction_ack.accepted);
    assert_eq!(interaction_ack.active_tab_id, "tab-2");
    assert_eq!(interaction_ack.interaction_epoch, 12);
    assert_eq!(
        interaction_ack.priority_order,
        vec![
            WorkKind::ForegroundDirectoryEnumeration,
            WorkKind::VisibleThumbnail,
            WorkKind::ForegroundSearch,
            WorkKind::FileProgress,
            WorkKind::BackgroundRefresh,
            WorkKind::NonVisibleThumbnail,
            WorkKind::BackgroundRecursiveSearch,
        ]
    );
    assert!(interaction_ack.interaction_latency_ms.is_some());
    assert!(!interaction_ack.frame_budget_degraded);
    assert!(interaction_ack.summary.contains("interaction ack"));

    let samples = observability::drain_performance_samples();
    assert!(
        samples
            .iter()
            .any(|sample| sample.key == observability::FRAME_BUDGET_DEGRADED_KEY),
        "viewport report should record frame_budget_degraded"
    );
    assert!(
        samples
            .iter()
            .any(|sample| sample.key == observability::INTERACTION_LATENCY_MS_KEY),
        "interaction report should record interaction_latency_ms"
    );
}

#[test]
fn scheduler_viewport_ack_marks_stale_epoch_for_old_signal() {
    let _guard = scheduler_test_guard();
    scheduler::reset_scheduler_state();
    observability::clear_performance_samples();

    let fresh_interaction = make_signal("tab-3", None, Some(10), false, 9);
    let _ = block_on_ready(commands::report_interaction_state(fresh_interaction))
        .expect("fresh interaction should succeed");

    let stale_viewport = make_signal("tab-3", None, Some(10), false, 8);
    let viewport_ack = block_on_ready(commands::report_viewport_state(stale_viewport))
        .expect("stale viewport report should still succeed");
    assert!(viewport_ack.stale_interaction_epoch);
    assert_eq!(viewport_ack.tracked_interaction_epoch, 9);
}
