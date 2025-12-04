use crate::{
    config::NegotiationConfig,
    negotiation::{
        Capabilities, ControlCaps, ControlFrame, Role, RolePreference, PROTOCOL_VERSION,
    },
    CACHE_DIR,
};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

/// Tracks the local node's handshake capabilities, node ID, and preference.
pub struct Negotiator {
    local_caps: Capabilities,
    preference: RolePreference,
    node_id: u32,
}

impl Negotiator {
    pub fn new(config: &NegotiationConfig) -> Self {
        Self {
            local_caps: Capabilities {
                supports_tunnel: true,
                supports_compression: false,
                supports_heartbeat: true,
            },
            preference: config.preference,
            node_id: config.node_id,
        }
    }

    pub fn hello_frame(&self) -> ControlFrame {
        ControlFrame::Hello {
            proto_version: PROTOCOL_VERSION,
            node_id: self.node_id,
            caps: ControlCaps {
                bits: self.local_caps.bits(),
            },
            pref: self.preference.as_str().to_string(),
        }
    }

    pub fn local_caps(&self) -> &Capabilities {
        &self.local_caps
    }

    pub fn decide_roles(&self, remote: &RemoteHello) -> NegotiationDecision {
        let local_rank = self.preference.priority_rank();
        let remote_rank = remote.preference.priority_rank();
        let local_wins_server = if local_rank != remote_rank {
            local_rank > remote_rank
        } else {
            self.node_id >= remote.node_id
        };
        let local_role = if local_wins_server {
            Role::Server
        } else {
            Role::Client
        };
        let remote_role = if local_wins_server {
            Role::Client
        } else {
            Role::Server
        };
        NegotiationDecision {
            local_role,
            remote_role,
        }
    }
}

/// Represents the paired role decisions for the local and remote peers.
pub struct NegotiationDecision {
    pub local_role: Role,
    pub remote_role: Role,
}

/// A parsed hello frame from the remote peer.
pub struct RemoteHello {
    pub node_id: u32,
    pub preference: RolePreference,
    pub capabilities: Capabilities,
}

impl RemoteHello {
    pub fn from_parts(node_id: u32, pref: &str, bits: u32) -> (Self, Option<String>) {
        match RolePreference::from_str(pref) {
            Ok(preference) => (
                Self {
                    node_id,
                    preference,
                    capabilities: Capabilities::from_bits(bits),
                },
                None,
            ),
            Err(reason) => (
                Self {
                    node_id,
                    preference: RolePreference::NoPreference,
                    capabilities: Capabilities::from_bits(bits),
                },
                Some(reason),
            ),
        }
    }
}

/// Logs negotiation states into `/run/serial_lcd_cache/logs/negotiation.log`.
pub struct NegotiationLog {
    file: Option<std::fs::File>,
}

impl NegotiationLog {
    pub fn try_create() -> std::io::Result<Self> {
        let log_path = Path::new(CACHE_DIR).join("logs").join("negotiation.log");
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(log_path)?;
        Ok(Self { file: Some(file) })
    }

    pub fn disabled() -> Self {
        Self { file: None }
    }

    pub fn record(&mut self, message: impl AsRef<str>) {
        if let Some(file) = self.file.as_mut() {
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs_f32())
                .unwrap_or(0.0);
            let _ = writeln!(file, "[{ts:.3}] {}", message.as_ref());
        }
    }
}
