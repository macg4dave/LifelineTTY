//! Heartbeat guard stub for the polling module. This will eventually emit watchdog packets and trigger
//! offline screens if missed.

/// Placeholder timer handle used to reset the heartbeat.
pub struct HeartbeatTimer;

impl HeartbeatTimer {
    /// Placeholder reset method documenting expected behavior.
    pub fn reset(&self) {
        // TODO: integrate with render loop timers.
    }
}
