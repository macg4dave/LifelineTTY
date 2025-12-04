use std::str::FromStr;

/// Negotiation/handshake skeleton for Milestone B / P9
/// This module defines a minimal capability bitmap and a small state machine
/// so other parts of the app can wire up negotiation behavior during testing.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Unknown,
    Server,
    Client,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Unknown => "unknown",
            Role::Server => "server",
            Role::Client => "client",
        }
    }
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "server" => Ok(Role::Server),
            "client" => Ok(Role::Client),
            "unknown" => Ok(Role::Unknown),
            other => Err(format!("invalid role '{other}', expected server|client")),
        }
    }
}

/// Local capability flags exchanged during the handshake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capabilities {
    pub supports_tunnel: bool,
    pub supports_compression: bool,
}

impl Capabilities {
    pub const HANDSHAKE_V1: u32 = 0b0000_0001;
    pub const CMD_TUNNEL_V1: u32 = 0b0000_0010;
    pub const LCD_V2: u32 = 0b0000_0100;
    pub const HEARTBEAT_V1: u32 = 0b0000_1000;
    pub const FILE_XFER_V1: u32 = 0b0001_0000;

    pub fn bits(&self) -> u32 {
        let mut bits = Self::HANDSHAKE_V1;
        if self.supports_tunnel {
            bits |= Self::CMD_TUNNEL_V1;
        }
        if self.supports_compression {
            bits |= Self::FILE_XFER_V1;
        }
        bits
    }

    pub fn from_bits(bits: u32) -> Self {
        Self {
            supports_tunnel: bits & Self::CMD_TUNNEL_V1 != 0,
            supports_compression: bits & Self::FILE_XFER_V1 != 0,
        }
    }
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            supports_tunnel: false,
            supports_compression: false,
        }
    }
}

/// Indicates which role the local node prefers during the handshake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RolePreference {
    PreferServer,
    PreferClient,
    NoPreference,
}

impl Default for RolePreference {
    fn default() -> Self {
        RolePreference::NoPreference
    }
}

impl RolePreference {
    pub fn as_str(&self) -> &'static str {
        match self {
            RolePreference::PreferServer => "prefer_server",
            RolePreference::PreferClient => "prefer_client",
            RolePreference::NoPreference => "no_preference",
        }
    }
}

impl FromStr for RolePreference {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "prefer_server" => Ok(RolePreference::PreferServer),
            "prefer_client" => Ok(RolePreference::PreferClient),
            "no_preference" | "none" => Ok(RolePreference::NoPreference),
            other => Err(format!("invalid preference '{other}'")),
        }
    }
}

pub struct Negotiator {
    role: Role,
    local: Capabilities,
    preference: RolePreference,
}

impl Negotiator {
    pub fn new(local_caps: Capabilities, preference: RolePreference) -> Self {
        Self {
            role: Role::Unknown,
            local: local_caps,
            preference,
        }
    }

    /// Start the negotiation and return the decided role and remote caps (stubbed)
    pub fn negotiate(&mut self, _remote_hello: &str) -> crate::Result<(Role, Capabilities)> {
        if self.local.supports_tunnel {
            self.role = Role::Server;
        } else {
            self.role = Role::Client;
        }
        Ok((self.role.clone(), Capabilities::default()))
    }

    pub fn set_role(&mut self, role: Role) {
        self.role = role;
    }

    pub fn role(&self) -> &Role {
        &self.role
    }

    pub fn local_caps(&self) -> &Capabilities {
        &self.local
    }

    pub fn preference(&self) -> RolePreference {
        self.preference
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negotiator_chooses_role() {
        let mut n = Negotiator::new(
            Capabilities {
                supports_tunnel: true,
                supports_compression: false,
            },
            RolePreference::default(),
        );
        let (role, caps) = n.negotiate("{}").unwrap();
        assert_eq!(role, Role::Server);
        assert_eq!(caps, Capabilities::default());
    }
}
