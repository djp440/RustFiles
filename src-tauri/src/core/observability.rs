use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const INTERACTION_LATENCY_MS_KEY: &str = "interaction_latency_ms";
pub const FRAME_BUDGET_DEGRADED_KEY: &str = "frame_budget_degraded";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PerformanceSample {
    pub key: String,
    pub value_ms: Option<u64>,
    pub degraded: Option<bool>,
    pub active_tab_id: String,
    pub interaction_epoch: u64,
    pub recorded_at_ms: u64,
}

static PERFORMANCE_SAMPLES: OnceLock<Mutex<Vec<PerformanceSample>>> = OnceLock::new();

fn samples() -> &'static Mutex<Vec<PerformanceSample>> {
    PERFORMANCE_SAMPLES.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn push_sample(sample: PerformanceSample) -> PerformanceSample {
    samples()
        .lock()
        .expect("performance samples lock poisoned")
        .push(sample.clone());
    sample
}

pub fn record_interaction_latency_ms(
    active_tab_id: &str,
    interaction_epoch: u64,
    value_ms: u64,
) -> PerformanceSample {
    push_sample(PerformanceSample {
        key: INTERACTION_LATENCY_MS_KEY.to_string(),
        value_ms: Some(value_ms),
        degraded: None,
        active_tab_id: active_tab_id.to_string(),
        interaction_epoch,
        recorded_at_ms: now_ms(),
    })
}

pub fn record_frame_budget_degraded(
    active_tab_id: &str,
    interaction_epoch: u64,
    degraded: bool,
) -> PerformanceSample {
    push_sample(PerformanceSample {
        key: FRAME_BUDGET_DEGRADED_KEY.to_string(),
        value_ms: None,
        degraded: Some(degraded),
        active_tab_id: active_tab_id.to_string(),
        interaction_epoch,
        recorded_at_ms: now_ms(),
    })
}

pub fn drain_performance_samples() -> Vec<PerformanceSample> {
    let mut guard = samples()
        .lock()
        .expect("performance samples lock poisoned");
    guard.drain(..).collect()
}

pub fn clear_performance_samples() {
    samples()
        .lock()
        .expect("performance samples lock poisoned")
        .clear();
}
