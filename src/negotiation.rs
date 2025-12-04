use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/// Protocol version shared by LifelineTTY endpoints during negotiation.
pub const PROTOCOL_VERSION: u8 = 1;

/// Role assigned to each peer after negotiation completes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Unknown,
    Server,
    Client,
}

impl Role {
    /// Human readable label used in logs and telemetry.
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Unknown => "unknown",
            Role::Server => "server",
            Role::Client => "client",
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Role::Server => Role::Client,
            Role::Client => Role::Server,
            Role::Unknown => Role::Unknown,
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "server" => Ok(Role::Server),
            "client" => Ok(Role::Client),
            "unknown" => Ok(Role::Unknown),
            other => Err(format!("invalid role '{other}', expected server|client")),
        }
    }
}

/// Preference hint used to nudge the election when both sides support negotiation.
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

    pub fn priority_rank(&self) -> u8 {
        match self {
            RolePreference::PreferServer => 2,
            RolePreference::NoPreference => 1,
            RolePreference::PreferClient => 0,
        }
    }
}

impl fmt::Display for RolePreference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for RolePreference {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "prefer_server" => Ok(RolePreference::PreferServer),
            "prefer_client" => Ok(RolePreference::PreferClient),
            "no_preference" | "none" => Ok(RolePreference::NoPreference),
            other => Err(format!("invalid preference '{other}'")),
        }
    }
}

/// Capability flags shared during the handshake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capabilities {
    pub supports_tunnel: bool,
    pub supports_compression: bool,
    pub supports_heartbeat: bool,
}

impl Capabilities {
    pub const COMPRESSION_V1: u32 = 0b0001_0000;
    pub const HANDSHAKE_V1: u32 = 0b0000_0001;
    pub const CMD_TUNNEL_V1: u32 = 0b0000_0010;
    pub const LCD_V2: u32 = 0b0000_0100;
    pub const HEARTBEAT_V1: u32 = 0b0000_1000;

    pub fn bits(&self) -> u32 {
        let mut bits = Self::HANDSHAKE_V1;
        if self.supports_tunnel {
            bits |= Self::CMD_TUNNEL_V1;
        }
        if self.supports_compression {
            bits |= Self::COMPRESSION_V1;
        }
        if self.supports_heartbeat {
            bits |= Self::HEARTBEAT_V1;
        }
        bits
    }

    pub fn from_bits(bits: u32) -> Self {
        Self {
            supports_tunnel: bits & Self::CMD_TUNNEL_V1 != 0,
            supports_compression: bits & Self::COMPRESSION_V1 != 0,
            supports_heartbeat: bits & Self::HEARTBEAT_V1 != 0,
        }
    }
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            supports_tunnel: false,
            supports_compression: false,
            supports_heartbeat: false,
        }
    }
}

/// Control-plane frames exchanged during negotiation.
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlFrame {
    Hello {
        proto_version: u8,
        node_id: u32,
        caps: ControlCaps,
        pref: String,
    },
    HelloAck {
        chosen_role: String,
        peer_caps: ControlCaps,
    },
    LegacyFallback,
}

/// Serialized wrapper for capability bits.
#[derive(Serialize, Deserialize)]
pub struct ControlCaps {
    pub bits: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compression_bit_round_trips() {
        let caps = Capabilities {
            supports_tunnel: false,
            supports_compression: true,
            supports_heartbeat: false,
        };
        let bits = caps.bits();
        assert!(bits & Capabilities::COMPRESSION_V1 != 0);
        let decoded = Capabilities::from_bits(bits);
        assert!(decoded.supports_compression);
        assert!(!decoded.supports_tunnel);
        assert!(!decoded.supports_heartbeat);
    }
}
