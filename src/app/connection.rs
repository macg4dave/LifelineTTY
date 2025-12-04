use super::Logger;
use crate::{
    app::negotiation::{NegotiationLog, Negotiator},
    config::NegotiationConfig,
    negotiation::{Capabilities, ControlCaps, ControlFrame, Role},
    serial::{classify_error, LineIo, SerialFailureKind, SerialOptions, SerialPort},
};
use serde_json;
use std::str::FromStr;
use std::time::{Duration, Instant};

struct NegotiationResult {
    role: Role,
    remote_role: Option<Role>,
    remote_caps: Option<Capabilities>,
    fallback: bool,
    pending_frame: Option<String>,
}

pub(crate) struct ConnectOutcome {
    pub port: SerialPort,
    pub remote_caps: Option<Capabilities>,
    pub fallback: bool,
    pub pending_frame: Option<String>,
}

/// Attempt to open the serial port, send the INIT handshake, and log outcomes.
pub(crate) fn attempt_serial_connect(
    logger: &Logger,
    device: &str,
    options: SerialOptions,
    negotiation: &NegotiationConfig,
    compression_enabled: bool,
    log: &mut NegotiationLog,
) -> Result<ConnectOutcome, SerialFailureKind> {
    match SerialPort::connect(device, options) {
        Ok(mut serial_connection) => {
            if let Err(err) = serial_connection.send_command_line("INIT") {
                let reason = classify_error(&err);
                logger.warn(format!("serial init failed [{reason}]: {err}; will retry"));
                return Err(reason);
            }
            logger.info("serial connected");
            log.record("negotiation: serial connected");
            let negotiation_result = negotiate_handshake(
                &mut serial_connection,
                logger,
                negotiation,
                compression_enabled,
                log,
            );
            if negotiation_result.fallback {
                logger.info("negotiation: falling back to legacy LCD-only mode");
                log.record("negotiation: falling back to legacy mode");
            } else {
                let caps_bits = negotiation_result
                    .remote_caps
                    .as_ref()
                    .map(|caps| caps.bits())
                    .unwrap_or(0);
                logger.info(format!(
                    "negotiation: role decided as {} remote_caps=0x{caps_bits:08x}",
                    negotiation_result.role.as_str()
                ));
                log.record(format!(
                    "negotiation: role={} remote_caps=0x{caps_bits:08x}",
                    negotiation_result.role.as_str()
                ));
            }
            Ok(ConnectOutcome {
                port: serial_connection,
                remote_caps: negotiation_result.remote_caps,
                fallback: negotiation_result.fallback,
                pending_frame: negotiation_result.pending_frame,
            })
        }
        Err(err) => {
            let reason = classify_error(&err);
            logger.warn(format!(
                "serial connect failed [{reason}]: {err}; will retry"
            ));
            Err(reason)
        }
    }
}

fn negotiate_handshake<IO>(
    io: &mut IO,
    logger: &Logger,
    config: &NegotiationConfig,
    compression_enabled: bool,
    log: &mut NegotiationLog,
) -> NegotiationResult
where
    IO: LineIo,
{
    let negotiator = Negotiator::new(config, compression_enabled);
    let hello_frame = negotiator.hello_frame();
    log.record("negotiation: sending hello");
    if !send_control_frame(io, &hello_frame, "hello", logger, log) {
        logger.warn("negotiation: failed to send hello frame");
        log.record("negotiation: failed to send hello frame");
        return fallback_result(None);
    }

    let deadline = Instant::now() + Duration::from_millis(config.timeout_ms);
    let mut buffer = String::new();

    while Instant::now() < deadline {
        match io.read_message_line(&mut buffer) {
            Ok(0) => continue,
            Ok(_) => {
                let trimmed = buffer.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match serde_json::from_str::<ControlFrame>(trimmed) {
                    Ok(ControlFrame::Hello {
                        node_id,
                        caps,
                        pref,
                        ..
                    }) => {
                        let (remote, pref_err) = crate::app::negotiation::RemoteHello::from_parts(
                            node_id, &pref, caps.bits,
                        );
                        if let Some(reason) = pref_err {
                            logger.warn(format!(
                                "negotiation: invalid preference '{pref}': {reason}"
                            ));
                            log.record(format!(
                                "negotiation: invalid preference '{pref}': {reason}"
                            ));
                        }
                        log.record(format!(
                            "negotiation: hello received node={} pref={} caps=0x{:08x}",
                            remote.node_id,
                            remote.preference.as_str(),
                            remote.capabilities.bits()
                        ));
                        let decision = negotiator.decide_roles(&remote);
                        let ack = ControlFrame::HelloAck {
                            chosen_role: decision.remote_role.as_str().to_string(),
                            peer_caps: ControlCaps {
                                bits: negotiator.local_caps().bits(),
                            },
                        };
                        if !send_control_frame(io, &ack, "hello_ack", logger, log) {
                            logger.warn("negotiation: failed to send hello_ack");
                            log.record("negotiation: failed to send hello_ack");
                            return fallback_result(None);
                        }
                        log.record(format!(
                            "negotiation: sent hello_ack remote_role={} local_role={}",
                            decision.remote_role.as_str(),
                            decision.local_role.as_str()
                        ));
                        continue;
                    }
                    Ok(ControlFrame::HelloAck {
                        chosen_role,
                        peer_caps,
                    }) => {
                        let role = Role::from_str(&chosen_role).unwrap_or(Role::Server);
                        let remote_role = role.opposite();
                        log.record(format!(
                            "negotiation: hello_ack received role={} caps=0x{:08x}",
                            role.as_str(),
                            peer_caps.bits
                        ));
                        return NegotiationResult {
                            role,
                            remote_role: Some(remote_role),
                            remote_caps: Some(Capabilities::from_bits(peer_caps.bits)),
                            fallback: false,
                            pending_frame: None,
                        };
                    }
                    Ok(ControlFrame::LegacyFallback) => {
                        log.record("negotiation: legacy_fallback received");
                        return fallback_result(None);
                    }
                    Err(_) => {
                        log.record(format!(
                            "negotiation: ignoring non-control frame during handshake: {trimmed}"
                        ));
                        return fallback_result(Some(trimmed.to_string()));
                    }
                }
            }
            Err(err) => {
                logger.warn(format!("negotiation: read failed: {err}"));
                log.record(format!("negotiation: read failed: {err}"));
                break;
            }
        }
    }

    log.record("negotiation: timed out".to_string());
    let _ = send_control_frame(
        io,
        &ControlFrame::LegacyFallback,
        "legacy_fallback",
        logger,
        log,
    );
    fallback_result(None)
}

fn fallback_result(pending_frame: Option<String>) -> NegotiationResult {
    NegotiationResult {
        role: Role::Server,
        remote_role: None,
        remote_caps: None,
        fallback: true,
        pending_frame,
    }
}

fn send_control_frame<IO>(
    io: &mut IO,
    frame: &ControlFrame,
    label: &str,
    logger: &Logger,
    log: &mut NegotiationLog,
) -> bool
where
    IO: LineIo,
{
    match serde_json::to_string(frame) {
        Ok(payload) => match io.send_command_line(&payload) {
            Ok(()) => {
                log.record(format!("negotiation: sent {label} frame"));
                true
            }
            Err(err) => {
                logger.warn(format!("negotiation: failed to send {label}: {err}"));
                log.record(format!("negotiation: failed to send {label}: {err}"));
                false
            }
        },
        Err(err) => {
            logger.warn(format!("negotiation: failed to encode {label}: {err}"));
            log.record(format!("negotiation: failed to encode {label}: {err}"));
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::logger::{LogLevel, Logger};
    use crate::serial::LineIo;
    use std::collections::VecDeque;

    struct FakeLineIo {
        responses: VecDeque<String>,
        sent: Vec<String>,
    }

    impl FakeLineIo {
        fn with_responses(responses: Vec<&str>) -> Self {
            Self {
                responses: responses
                    .into_iter()
                    .map(String::from)
                    .collect::<VecDeque<_>>(),
                sent: Vec::new(),
            }
        }

        fn sent(&self) -> &[String] {
            &self.sent
        }
    }

    impl LineIo for FakeLineIo {
        fn send_command_line(&mut self, line: &str) -> crate::Result<()> {
            self.sent.push(line.to_string());
            Ok(())
        }

        fn read_message_line(&mut self, buf: &mut String) -> crate::Result<usize> {
            if let Some(line) = self.responses.pop_front() {
                buf.clear();
                buf.push_str(&line);
                return Ok(line.len());
            }
            Ok(0)
        }
    }

    fn new_logger() -> Logger {
        Logger::new(LogLevel::Debug, None).expect("logger init")
    }

    #[test]
    fn negotiation_success_sets_role() {
        let ack = r#"{"type":"hello_ack","chosen_role":"client","peer_caps":{"bits":3}}"#;
        let mut io = FakeLineIo::with_responses(vec![ack]);
        let logger = new_logger();
        let mut log = NegotiationLog::disabled();
        let result = negotiate_handshake(
            &mut io,
            &logger,
            &NegotiationConfig::default(),
            false,
            &mut log,
        );
        assert!(!result.fallback);
        assert_eq!(result.role, Role::Client);
        assert_eq!(result.remote_role, Some(Role::Server));
        assert!(io
            .sent()
            .iter()
            .any(|line| line.contains("\"type\":\"hello\"")));
    }

    #[test]
    fn negotiation_hello_triggers_ack_and_success() {
        let hello = r#"{"type":"hello","proto_version":1,"node_id":99,"caps":{"bits":2},"pref":"prefer_server"}"#;
        let ack = r#"{"type":"hello_ack","chosen_role":"client","peer_caps":{"bits":2}}"#;
        let mut io = FakeLineIo::with_responses(vec![hello, ack]);
        let logger = new_logger();
        let mut log = NegotiationLog::disabled();
        let result = negotiate_handshake(
            &mut io,
            &logger,
            &NegotiationConfig::default(),
            false,
            &mut log,
        );
        assert!(!result.fallback);
        assert!(io
            .sent()
            .iter()
            .any(|line| line.contains("\"type\":\"hello_ack\"")));
    }

    #[test]
    fn negotiation_unknown_frame_promotes_fallback_with_frame() {
        let unknown = r#"{"payload":"render"}"#;
        let mut io = FakeLineIo::with_responses(vec![unknown]);
        let logger = new_logger();
        let mut log = NegotiationLog::disabled();
        let result = negotiate_handshake(
            &mut io,
            &logger,
            &NegotiationConfig::default(),
            false,
            &mut log,
        );
        assert!(result.fallback);
        assert_eq!(result.pending_frame.as_deref(), Some(unknown));
    }
}
