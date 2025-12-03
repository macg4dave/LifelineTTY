//! State machine skeleton for negotiation. Track handshakes, role resolution, and timeout
//! transitions to keep the LCD from blanking during negotiation.

/// Placeholder state enum for the negotiation flow.
pub enum NegotiationState {
    Idle,
    Negotiating,
    Resolved,
}

impl NegotiationState {
    /// Placeholder transition that would run after a timeout expires.
    pub fn timeout(&mut self) {
        *self = NegotiationState::Idle;
    }
}
