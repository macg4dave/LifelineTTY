use crate::{Error, Result};

/// Negotiation/handshake skeleton for Milestone B / P9
/// This module defines a minimal capability bitmap and a small state machine
/// so other parts of the app can wire up negotiation behavior during testing.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Unknown,
    Server,
    Client,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capabilities {
    pub supports_tunnel: bool,
    pub supports_compression: bool,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            supports_tunnel: false,
            supports_compression: false,
        }
    }
}

pub struct Negotiator {
    role: Role,
    local: Capabilities,
}

impl Negotiator {
    pub fn new(local_caps: Capabilities) -> Self {
        Self {
            role: Role::Unknown,
            local: local_caps,
        }
    }

    /// Start the negotiation and return the decided role and remote caps (stubbed)
    pub fn negotiate(&mut self, _remote_hello: &str) -> Result<(Role, Capabilities)> {
        // Minimal deterministic behavior for skeleton: become Server if local supports tunnel
        if self.local.supports_tunnel {
            self.role = Role::Server;
        } else {
            self.role = Role::Client;
        }
        Ok((self.role.clone(), Capabilities::default()))
    }

    pub fn role(&self) -> &Role {
        &self.role
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negotiator_chooses_role() {
        let mut n = Negotiator::new(Capabilities {
            supports_tunnel: true,
            supports_compression: false,
        });
        let (role, caps) = n.negotiate("{}").unwrap();
        assert_eq!(role, Role::Server);
        assert_eq!(caps, Capabilities::default());
    }
}
