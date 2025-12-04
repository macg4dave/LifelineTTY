use super::Logger;
use crate::app::negotiation::{Capabilities, Negotiator, Role, RolePreference};
use crate::serial::{backoff::BackoffController, LineIo, SerialOptions, SerialPort};
use serde::Deserialize;
use serde_json::json;
use std::{
    str::FromStr,
    time::{Duration, Instant},
};

pub(crate) struct SerialConnection {
    pub port: SerialPort,
    pub initial_frame: Option<String>,
    pub negotiated_role: Role,
}

struct NegotiationResult {
    role: Role,
    pending_frame: Option<String>,
    fallback: bool,
}

const NEGOTIATION_TIMEOUT_MS: u64 = 500;
const NEGOTIATION_NODE_ID: u32 = 42;
const NEGOTIATION_PROTO_VERSION: u8 = 1;

/// Attempt to open the serial port and send the INIT handshake, logging outcomes.
pub(crate) fn attempt_serial_connect(
    logger: &Logger,
    device: &str,
    options: SerialOptions,
) -> Option<SerialConnection> {
    match SerialPort::connect(device, options) {
        Ok(mut serial_connection) => {
            if let Err(err) = serial_connection.send_command_line("INIT") {
                logger.warn(format!("serial init failed: {err}; will retry"));
                return None;
            }
            logger.info("serial connected");
            let negotiation = negotiate_handshake(&mut serial_connection, logger);
            if negotiation.fallback {
                logger.info("negotiation: falling back to legacy LCD-only mode");
            } else {
                logger.info(format!(
                    "negotiation: role decided as {}",
                    negotiation.role.as_str()
                ));
            }
            Some(SerialConnection {
                port: serial_connection,
                initial_frame: negotiation.pending_frame,
                negotiated_role: negotiation.role,
            })
        }
        Err(err) => {
            logger.warn(format!("serial connect failed: {err}; will retry"));
            None
        }
    }
}

fn negotiate_handshake<IO>(io: &mut IO, logger: &Logger) -> NegotiationResult
where
    IO: LineIo,
{
    let mut negotiator = Negotiator::new(
        Capabilities {
            supports_tunnel: true,
            supports_compression: false,
        },
        RolePreference::PreferServer,
    );
    let hello = json!({
        "type": "hello",
        "proto_version": NEGOTIATION_PROTO_VERSION,
        "node_id": NEGOTIATION_NODE_ID,
        "caps": { "bits": negotiator.local_caps().bits() },
        "pref": negotiator.preference().as_str(),
    });
    if io.send_command_line(&hello.to_string()).is_err() {
        logger.warn("negotiation: failed to write hello frame");
        return fallback_result();
    }

    let deadline = Instant::now() + Duration::from_millis(NEGOTIATION_TIMEOUT_MS);
    let mut buffer = String::new();

    while Instant::now() < deadline {
        match io.read_message_line(&mut buffer) {
            Ok(0) => continue,
            Ok(_) => {
                let trimmed = buffer.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match parse_control_frame(trimmed) {
                    Some(ControlFrame::HelloAck {
                        chosen_role,
                        peer_caps: _peer_caps,
                    }) => {
                        let role = Role::from_str(&chosen_role).unwrap_or(Role::Server);
                        negotiator.set_role(role.clone());
                        logger.info("negotiation: received hello_ack");
                        return NegotiationResult {
                            role,
                            pending_frame: None,
                            fallback: false,
                        };
                    }
                    Some(ControlFrame::LegacyFallback) => {
                        logger.info("negotiation: peer requested legacy fallback");
                        break;
                    }
                    Some(ControlFrame::Hello { .. }) => continue,
                    None => {
                        return NegotiationResult {
                            role: Role::Server,
                            pending_frame: Some(trimmed.to_string()),
                            fallback: true,
                        };
                    }
                }
            }
            Err(err) => {
                logger.warn(format!("negotiation: read failed: {err}"));
                break;
            }
        }
    }

    fallback_result()
}

fn fallback_result() -> NegotiationResult {
    NegotiationResult {
        role: Role::Server,
        pending_frame: None,
        fallback: true,
    }
}

fn parse_control_frame(raw: &str) -> Option<ControlFrame> {
    serde_json::from_str(raw).ok()
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ControlFrame {
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

#[derive(Deserialize)]
struct ControlCaps {
    bits: u32,
}
