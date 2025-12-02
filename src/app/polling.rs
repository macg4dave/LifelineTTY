use crate::Result;
use std::time::Duration;

/// Lightweight polling helper for future Milestone D work.
/// This is intentionally a no-op / stub for roadmap skeletons so higher-level
/// work can call into it without dependency churn.
pub struct Poller {
    interval: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PollSnapshot {
    pub cpu_percent: f32,
    pub mem_kb: u64,
}

impl Poller {
    pub fn new(interval_ms: u64) -> Self {
        Self {
            interval: Duration::from_millis(interval_ms),
        }
    }

    /// Perform one poll cycle. Currently returns a deterministic stub value.
    pub fn poll_once(&self) -> Result<PollSnapshot> {
        // Stubbed values: real implementation will read /proc or use a light crate
        Ok(PollSnapshot {
            cpu_percent: 0.0,
            mem_kb: 0,
        })
    }

    pub fn interval_ms(&self) -> u64 {
        self.interval.as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poller_constructs_and_polls() {
        let p = Poller::new(1000);
        assert_eq!(p.interval_ms(), 1000);
        let snap = p.poll_once().unwrap();
        assert_eq!(snap.cpu_percent, 0.0);
        assert_eq!(snap.mem_kb, 0);
    }
}
