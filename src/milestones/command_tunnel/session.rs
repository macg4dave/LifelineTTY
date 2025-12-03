//! Command session state machine stub for the command tunnel. Once the executor arrives, this module
//! will track `Idle`, `Running`, and `Busy` states, enforcing a single active command.

/// Rough placeholder for future session states.
pub enum SessionState {
    Idle,
    Running,
    Busy,
}

impl SessionState {
    /// Placeholder method describing how the FSM will transition when a command starts.
    pub fn enter_running(&mut self) {
        *self = SessionState::Running;
    }
}
