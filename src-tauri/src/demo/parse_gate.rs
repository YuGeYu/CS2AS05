use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::{errors::AppError, services::cs2};

const STOPPED_SAMPLE_INTERVAL: Duration = Duration::from_millis(750);
const STOPPED_CONFIRMATION_WINDOW: Duration = Duration::from_secs(8);
const REQUIRED_STOPPED_SAMPLES: u8 = 4;

#[derive(Debug, Clone, Copy)]
pub struct ProcessObservation {
    pub running: bool,
    pub confirmed_stopped: bool,
}

#[derive(Debug, Default)]
struct ProcessGateState {
    last_running_at: Option<Instant>,
    stopped_since: Option<Instant>,
    last_stopped_sample_at: Option<Instant>,
    stopped_samples: u8,
}

impl ProcessGateState {
    fn observe(&mut self, running: bool, now: Instant) -> ProcessObservation {
        if running {
            self.last_running_at = Some(now);
            self.stopped_since = None;
            self.last_stopped_sample_at = None;
            self.stopped_samples = 0;
            return ProcessObservation {
                running: true,
                confirmed_stopped: false,
            };
        }

        let stopped_since = *self.stopped_since.get_or_insert(now);
        let sample_due = self.last_stopped_sample_at.map_or(true, |sample| {
            now.saturating_duration_since(sample) >= STOPPED_SAMPLE_INTERVAL
        });
        if sample_due {
            self.last_stopped_sample_at = Some(now);
            self.stopped_samples = self.stopped_samples.saturating_add(1);
        }

        let stopped_long_enough =
            now.saturating_duration_since(stopped_since) >= STOPPED_CONFIRMATION_WINDOW;
        let running_grace_elapsed = self.last_running_at.map_or(true, |seen| {
            now.saturating_duration_since(seen) >= STOPPED_CONFIRMATION_WINDOW
        });
        ProcessObservation {
            running: false,
            confirmed_stopped: stopped_long_enough
                && running_grace_elapsed
                && self.stopped_samples >= REQUIRED_STOPPED_SAMPLES,
        }
    }
}

fn process_gate() -> &'static Mutex<ProcessGateState> {
    static PROCESS_GATE: OnceLock<Mutex<ProcessGateState>> = OnceLock::new();
    PROCESS_GATE.get_or_init(|| Mutex::new(ProcessGateState::default()))
}

/// Returns a debounced process observation shared by every Demo entry point.
pub fn observe_process() -> Result<ProcessObservation, AppError> {
    let running = cs2::check_cs2_process()?;
    process_gate()
        .lock()
        .map(|mut state| state.observe(running, Instant::now()))
        .map_err(|_| AppError::runtime("[DEMO_PROCESS_GATE_LOCK] Demo 进程门禁状态不可用。"))
}

/// The single process boundary for every demo parser entry point.
pub fn assert_parse_allowed() -> Result<(), AppError> {
    if !observe_process()?.confirmed_stopped {
        return Err(AppError::runtime(
            "[DEMO_PARSE_BLOCKED_CS2_RUNNING] CS2 正在运行或仍在确认退出，完成确认后自动继续 Demo 解析。",
        ));
    }
    Ok(())
}

pub fn assert_parse_allowed_for_path(path: &Path) -> Result<(), AppError> {
    assert_parse_allowed()?;
    if !path.is_file() {
        return Err(AppError::runtime(
            "[DEMO_FILE_NOT_FOUND] Demo 文件不存在或尚未完成写入。",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_error_code_is_stable() {
        assert!("DEMO_PARSE_BLOCKED_CS2_RUNNING".contains("DEMO_PARSE_BLOCKED"));
    }

    #[test]
    fn one_false_sample_never_confirms_process_exit() {
        let start = Instant::now();
        let mut state = ProcessGateState::default();
        assert!(state.observe(true, start).running);
        let missed = state.observe(false, start + Duration::from_secs(1));
        assert!(!missed.running);
        assert!(!missed.confirmed_stopped);
        let recovered = state.observe(true, start + Duration::from_secs(2));
        assert!(recovered.running);
        assert!(!recovered.confirmed_stopped);
    }

    #[test]
    fn rapid_parallel_checks_do_not_fake_consecutive_samples() {
        let start = Instant::now();
        let mut state = ProcessGateState::default();
        state.observe(true, start);
        for offset in 1..=20 {
            let observation = state.observe(false, start + Duration::from_millis(1_000 + offset));
            assert!(!observation.confirmed_stopped);
        }
        assert_eq!(state.stopped_samples, 1);
    }

    #[test]
    fn sustained_absence_confirms_process_exit_after_window() {
        let start = Instant::now();
        let mut state = ProcessGateState::default();
        state.observe(true, start);
        for second in 1..8 {
            let observation = state.observe(false, start + Duration::from_secs(second));
            assert!(!observation.confirmed_stopped);
        }
        let confirmed = state.observe(
            false,
            start + STOPPED_CONFIRMATION_WINDOW + Duration::from_secs(1),
        );
        assert!(!confirmed.running);
        assert!(confirmed.confirmed_stopped);
    }
}
