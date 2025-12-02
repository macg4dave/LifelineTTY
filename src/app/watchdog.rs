use std::time::{Duration, Instant};

/// Heartbeat / watchdog skeleton for Milestone P15.
pub struct Watchdog {
    last_seen: Instant,
    timeout: Duration,
}

impl Watchdog {
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            last_seen: Instant::now(),
            timeout: Duration::from_millis(timeout_ms),
        }
    }

    pub fn touch(&mut self) {
        self.last_seen = Instant::now();
    }

    pub fn is_expired(&self) -> bool {
        self.last_seen.elapsed() > self.timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn watchdog_expires_and_resets() {
        let mut w = Watchdog::new(5);
        sleep(Duration::from_millis(10));
        assert!(w.is_expired());
        w.touch();
        assert!(!w.is_expired());
    }
}
