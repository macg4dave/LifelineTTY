use crate::{app::Logger, CACHE_DIR};
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

const WATCHDOG_DIR: &str = "watchdog";
const HOOK_NAME: &str = "offline_hook.sh";

/// Tracks last-seen timestamps for watchdog channels.
#[derive(Debug, Clone)]
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

    pub fn is_expired_at(&self, now: Instant) -> bool {
        now.duration_since(self.last_seen) > self.timeout
    }
}

/// Describes state transitions for watchdog channels.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WatchdogStatus {
    pub serial_expired: bool,
    pub tunnel_expired: bool,
    pub serial_recovered: bool,
    pub tunnel_recovered: bool,
}

struct WatchdogLog {
    path: PathBuf,
}

impl WatchdogLog {
    fn new() -> Self {
        let path = PathBuf::from(CACHE_DIR)
            .join(WATCHDOG_DIR)
            .join("events.log");
        Self { path }
    }

    fn append(&self, line: &str) {
        if let Some(parent) = self.path.parent() {
            if let Err(err) = create_dir_all(parent) {
                eprintln!("watchdog log mkdir failed: {err}");
                return;
            }
        }
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let _ = writeln!(file, "{line}");
        }
    }
}

/// Multi-channel watchdog monitor with optional offline hook.
pub struct WatchdogMonitor {
    serial: Watchdog,
    tunnel: Watchdog,
    serial_expired: bool,
    tunnel_expired: bool,
    hook_invoked: bool,
    log: WatchdogLog,
}

impl WatchdogMonitor {
    pub fn new(serial_timeout_ms: u64, tunnel_timeout_ms: u64) -> Self {
        Self {
            serial: Watchdog::new(serial_timeout_ms),
            tunnel: Watchdog::new(tunnel_timeout_ms),
            serial_expired: false,
            tunnel_expired: false,
            hook_invoked: false,
            log: WatchdogLog::new(),
        }
    }

    pub fn touch_serial(&mut self) {
        self.serial.touch();
    }

    pub fn touch_tunnel(&mut self) {
        self.tunnel.touch();
    }

    /// Evaluate watchdogs and emit transition status.
    pub fn evaluate(&mut self, logger: &Logger) -> WatchdogStatus {
        let now = Instant::now();
        let mut status = WatchdogStatus::default();

        if self.serial.is_expired_at(now) {
            if !self.serial_expired {
                self.serial_expired = true;
                self.log.append("serial_expired");
            }
            status.serial_expired = true;
        } else if self.serial_expired {
            status.serial_recovered = true;
            self.serial_expired = false;
            self.hook_invoked = false;
            self.log.append("serial_recovered");
        }

        if self.tunnel.is_expired_at(now) {
            if !self.tunnel_expired {
                self.tunnel_expired = true;
                self.log.append("tunnel_expired");
            }
            status.tunnel_expired = true;
        } else if self.tunnel_expired {
            status.tunnel_recovered = true;
            self.tunnel_expired = false;
            self.hook_invoked = false;
            self.log.append("tunnel_recovered");
        }

        if (status.serial_expired || status.tunnel_expired) && !self.hook_invoked {
            self.trigger_hook(logger);
            self.hook_invoked = true;
        }

        status
    }

    fn trigger_hook(&self, logger: &Logger) {
        let hook_path = PathBuf::from(CACHE_DIR).join(WATCHDOG_DIR).join(HOOK_NAME);
        if !hook_path.exists() {
            logger.debug("watchdog: offline hook missing; skipping");
            return;
        }
        let log_path = self.log.path.clone();
        logger.warn(format!(
            "watchdog: triggering offline hook {}",
            hook_path.display()
        ));
        thread::spawn(move || {
            let result = Command::new(&hook_path).output();
            let log_line = match result {
                Ok(output) => {
                    let code = output.status.code().unwrap_or(-1);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    format!(
                        "hook status={} stdout={} stderr={}",
                        code,
                        stdout.trim(),
                        stderr.trim()
                    )
                }
                Err(err) => format!("hook failed to run: {err}"),
            };
            if let Some(parent) = log_path.parent() {
                let _ = create_dir_all(parent);
            }
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_path) {
                let _ = writeln!(file, "{log_line}");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::logger::{LogLevel, Logger};
    use std::thread::sleep;

    #[test]
    fn watchdog_expires_and_resets() {
        let mut w = Watchdog::new(5);
        sleep(Duration::from_millis(10));
        assert!(w.is_expired_at(Instant::now()));
        w.touch();
        assert!(!w.is_expired_at(Instant::now()));
    }

    #[test]
    fn monitor_tracks_transitions() {
        let logger = Logger::new(LogLevel::Debug, None).unwrap();
        let mut monitor = WatchdogMonitor::new(5, 5);
        sleep(Duration::from_millis(10));
        let status = monitor.evaluate(&logger);
        assert!(status.serial_expired);
        assert!(status.tunnel_expired);

        monitor.touch_serial();
        monitor.touch_tunnel();
        let recovered = monitor.evaluate(&logger);
        assert!(recovered.serial_recovered);
        assert!(recovered.tunnel_recovered);
    }
}
