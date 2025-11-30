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

    /// Record a failure and push the next retry into the future with backoff + jitter.
    pub fn mark_failure(&mut self, now: Instant) {
        let jitter = self.jitter(self.current);
        self.next_retry_at = now + self.current + jitter;
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

    pub fn current_delay_ms(&self) -> u64 {
        self.current.as_millis() as u64
    }

    pub fn max_delay_ms(&self) -> u64 {
        self.max.as_millis() as u64
    }

    fn jitter(&self, base: Duration) -> Duration {
        use std::time::SystemTime;
        let millis = base.as_millis() as u64;
        if millis == 0 {
            return Duration::from_millis(0);
        }
        let cap = (millis / 4).max(1);
        let seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.subsec_millis() as u64)
            .unwrap_or(0);
        let jitter = seed % cap;
        Duration::from_millis(jitter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_grows_with_jitter_and_caps() {
        let mut b = BackoffController::new(100, 400);
        let now = Instant::now();
        b.mark_failure(now);
        assert!(b.current_delay_ms() >= 100);
        b.mark_failure(now);
        assert!(b.current_delay_ms() <= 400);
        assert!(b.max_delay_ms() == 400);
    }

    #[test]
    fn backoff_resets_on_success() {
        let mut b = BackoffController::new(200, 800);
        let now = Instant::now();
        b.mark_failure(now);
        b.mark_failure(now);
        assert!(b.current_delay_ms() >= 200);
        b.mark_success(now);
        assert_eq!(b.current_delay_ms(), 200);
    }
}
