use std::time::{Duration, Instant};

/// Tracks reconnect backoff timing and schedules the next retry window.
pub struct BackoffController {
    initial: Duration,
    max: Duration,
    current: Duration,
    next_retry_at: Instant,
}

impl BackoffController {
    pub fn new(initial_ms: u64, max_ms: u64) -> Self {
        let initial = Duration::from_millis(initial_ms.max(1));
        let max = Duration::from_millis(max_ms.max(initial_ms.max(1)));
        Self {
            initial,
            max,
            current: initial,
            next_retry_at: Instant::now(),
        }
    }

    /// Record a failure and push the next retry into the future with backoff.
    pub fn mark_failure(&mut self, now: Instant) {
        self.next_retry_at = now + self.current;
        self.current = (self.current * 2).min(self.max);
    }

    /// Reset backoff after a successful connect attempt.
    pub fn mark_success(&mut self, now: Instant) {
        self.current = self.initial;
        self.next_retry_at = now;
    }

    pub fn should_retry(&self, now: Instant) -> bool {
        now >= self.next_retry_at
    }

    pub fn update(&mut self, initial_ms: u64, max_ms: u64) {
        let initial = Duration::from_millis(initial_ms.max(1));
        let max = Duration::from_millis(max_ms.max(initial_ms.max(1)));
        self.initial = initial;
        self.max = max;
        self.current = initial;
        self.next_retry_at = Instant::now();
    }
}
