use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};

use crate::core::observability;

/// UI 优先调度的可视范围。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct VisibleRange {
    pub start_index: usize,
    pub end_index: usize,
}

impl VisibleRange {
    pub fn span(&self) -> usize {
        self.end_index.saturating_sub(self.start_index).saturating_add(1)
    }
}

/// 前端上报的最小调度信号。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SchedulerSignal {
    pub active_tab_id: String,
    pub visible_range: Option<VisibleRange>,
    pub last_input_at_ms: Option<u64>,
    pub is_scrolling: bool,
    pub interaction_epoch: u64,
}

impl SchedulerSignal {
    pub fn visible_span(&self) -> Option<usize> {
        self.visible_range.as_ref().map(VisibleRange::span)
    }

    pub fn interaction_latency_ms(&self, now_ms: u64) -> Option<u64> {
        self.last_input_at_ms.map(|last_input_at_ms| now_ms.saturating_sub(last_input_at_ms))
    }
}

/// 调度工作类型。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WorkKind {
    ForegroundDirectoryEnumeration,
    VisibleThumbnail,
    ForegroundSearch,
    FileProgress,
    BackgroundRefresh,
    NonVisibleThumbnail,
    BackgroundRecursiveSearch,
}

impl WorkKind {
    fn sort_index(&self) -> u8 {
        match self {
            Self::ForegroundDirectoryEnumeration => 0,
            Self::VisibleThumbnail => 1,
            Self::ForegroundSearch => 2,
            Self::FileProgress => 3,
            Self::BackgroundRefresh => 4,
            Self::NonVisibleThumbnail => 5,
            Self::BackgroundRecursiveSearch => 6,
        }
    }
}

/// 调度优先级分层。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PriorityKind {
    Urgent,
    High,
    Normal,
    Low,
    Background,
    Deferred,
    Lowest,
}

/// 单个工作项的优先级快照。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ScheduledWork {
    pub work_kind: WorkKind,
    pub priority_kind: PriorityKind,
    pub score: u16,
}

/// 调度上报类型。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SchedulerReportKind {
    Viewport,
    Interaction,
}

/// 调度 command 返回的最小 ack。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SchedulerReportAck {
    pub accepted: bool,
    pub report_kind: SchedulerReportKind,
    pub active_tab_id: String,
    pub interaction_epoch: u64,
    pub visible_range: Option<VisibleRange>,
    pub priority_order: Vec<WorkKind>,
    pub interaction_latency_ms: Option<u64>,
    pub frame_budget_degraded: bool,
    pub stale_interaction_epoch: bool,
    pub tracked_interaction_epoch: u64,
    pub summary: String,
}

#[derive(Default)]
struct TabSnapshot {
    latest_interaction_epoch: u64,
}

static TAB_SNAPSHOTS: OnceLock<Mutex<HashMap<String, TabSnapshot>>> = OnceLock::new();

fn snapshots() -> &'static Mutex<HashMap<String, TabSnapshot>> {
    TAB_SNAPSHOTS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn update_tab_snapshot(active_tab_id: &str, interaction_epoch: u64) -> (bool, u64) {
    let mut snapshots = snapshots().lock().expect("scheduler snapshots lock poisoned");
    let entry = snapshots
        .entry(active_tab_id.to_string())
        .or_insert_with(TabSnapshot::default);
    let stale = interaction_epoch < entry.latest_interaction_epoch;
    if !stale {
        entry.latest_interaction_epoch = interaction_epoch;
    }
    (stale, entry.latest_interaction_epoch)
}

fn priority_score(signal: &SchedulerSignal, work_kind: &WorkKind) -> u16 {
    let visible_span = signal.visible_span().unwrap_or(0);
    match work_kind {
        WorkKind::ForegroundDirectoryEnumeration => 0,
        WorkKind::VisibleThumbnail => {
            if visible_span > 0 {
                10
            } else {
                12
            }
        }
        WorkKind::ForegroundSearch => 20,
        WorkKind::FileProgress => 30,
        WorkKind::BackgroundRefresh => {
            if signal.is_scrolling {
                50
            } else {
                40
            }
        }
        WorkKind::NonVisibleThumbnail => {
            if signal.is_scrolling {
                110
            } else {
                50
            }
        }
        WorkKind::BackgroundRecursiveSearch => {
            if signal.is_scrolling {
                130
            } else {
                60
            }
        }
    }
}

fn priority_kind_for_score(score: u16) -> PriorityKind {
    match score {
        0..=9 => PriorityKind::Urgent,
        10..=19 => PriorityKind::High,
        20..=29 => PriorityKind::Normal,
        30..=39 => PriorityKind::Low,
        40..=59 => PriorityKind::Background,
        60..=99 => PriorityKind::Deferred,
        _ => PriorityKind::Lowest,
    }
}

pub fn is_interaction_epoch_stale(latest_epoch: u64, incoming_epoch: u64) -> bool {
    incoming_epoch < latest_epoch
}

pub fn reset_scheduler_state() {
    snapshots()
        .lock()
        .expect("scheduler snapshots lock poisoned")
        .clear();
}

pub fn priority_snapshot(signal: &SchedulerSignal) -> Vec<ScheduledWork> {
    let mut snapshot: Vec<ScheduledWork> = vec![
        WorkKind::ForegroundDirectoryEnumeration,
        WorkKind::VisibleThumbnail,
        WorkKind::ForegroundSearch,
        WorkKind::FileProgress,
        WorkKind::BackgroundRefresh,
        WorkKind::NonVisibleThumbnail,
        WorkKind::BackgroundRecursiveSearch,
    ]
    .into_iter()
    .map(|work_kind| {
        let score = priority_score(signal, &work_kind);
        ScheduledWork {
            work_kind,
            priority_kind: priority_kind_for_score(score),
            score,
        }
    })
    .collect();

    snapshot.sort_by(|left, right| {
        left.score
            .cmp(&right.score)
            .then_with(|| left.work_kind.sort_index().cmp(&right.work_kind.sort_index()))
    });
    snapshot
}

pub fn priority_order(signal: &SchedulerSignal) -> Vec<WorkKind> {
    priority_snapshot(signal)
        .into_iter()
        .map(|scheduled| scheduled.work_kind)
        .collect()
}

fn summarize_report(
    signal: SchedulerSignal,
    report_kind: SchedulerReportKind,
    interaction_latency_ms: Option<u64>,
    frame_budget_degraded: bool,
) -> SchedulerReportAck {
    let priority_order = priority_order(&signal);
    let (stale_interaction_epoch, tracked_interaction_epoch) =
        update_tab_snapshot(&signal.active_tab_id, signal.interaction_epoch);

    observability::record_frame_budget_degraded(
        &signal.active_tab_id,
        signal.interaction_epoch,
        frame_budget_degraded,
    );

    let visible_range = signal.visible_range.clone();
    let summary = match report_kind {
        SchedulerReportKind::Viewport => format!(
            "viewport ack for tab '{}' with {} priority tiers{}{}",
            signal.active_tab_id,
            priority_order.len(),
            if frame_budget_degraded {
                ", frame budget degraded"
            } else {
                ""
            },
            if stale_interaction_epoch {
                ", stale interaction epoch detected"
            } else {
                ""
            }
        ),
        SchedulerReportKind::Interaction => format!(
            "interaction ack for tab '{}' with latency {}{}",
            signal.active_tab_id,
            interaction_latency_ms
                .map(|value| format!("{value}ms"))
                .unwrap_or_else(|| "unknown".to_string()),
            if frame_budget_degraded {
                ", frame budget degraded"
            } else {
                ""
            }
        ),
    };

    SchedulerReportAck {
        accepted: true,
        report_kind,
        active_tab_id: signal.active_tab_id,
        interaction_epoch: signal.interaction_epoch,
        visible_range,
        priority_order,
        interaction_latency_ms,
        frame_budget_degraded,
        stale_interaction_epoch,
        tracked_interaction_epoch,
        summary,
    }
}

pub fn summarize_viewport_state(signal: SchedulerSignal) -> SchedulerReportAck {
    let frame_budget_degraded = signal.is_scrolling
        || signal
            .visible_span()
            .map(|span| span >= 128)
            .unwrap_or(false);
    let interaction_latency_ms = signal.interaction_latency_ms(observability::now_ms());
    summarize_report(
        signal,
        SchedulerReportKind::Viewport,
        interaction_latency_ms,
        frame_budget_degraded,
    )
}

pub fn summarize_interaction_state(signal: SchedulerSignal) -> SchedulerReportAck {
    let interaction_latency_ms = signal.interaction_latency_ms(observability::now_ms());
    if let Some(latency_ms) = interaction_latency_ms {
        observability::record_interaction_latency_ms(
            &signal.active_tab_id,
            signal.interaction_epoch,
            latency_ms,
        );
    }

    let frame_budget_degraded = signal.is_scrolling;
    summarize_report(
        signal,
        SchedulerReportKind::Interaction,
        interaction_latency_ms,
        frame_budget_degraded,
    )
}
