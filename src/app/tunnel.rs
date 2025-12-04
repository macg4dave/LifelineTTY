use super::Logger;
use crate::app::events::{CommandEvent, CommandExecutor};
use crate::{
    payload::{CommandMessage, CommandStream, TunnelMsgOwned},
    Result, CACHE_DIR,
};
use std::fs::{create_dir_all, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TunnelController {
    executor: CommandExecutor,
    request_counter: AtomicU32,
    tunnel_dir: PathBuf,
}

impl TunnelController {
    pub fn new(allowlist: Vec<String>) -> Result<Self> {
        let tunnel_dir = PathBuf::from(CACHE_DIR).join("tunnel");
        match create_dir_all(&tunnel_dir) {
            Ok(_) => {}
            Err(err)
                if matches!(
                    err.kind(),
                    ErrorKind::PermissionDenied | ErrorKind::ReadOnlyFilesystem
                ) =>
            {
                // Best effort in environments where CACHE_DIR is read-only (e.g., tests)
            }
            Err(err) => return Err(err.into()),
        }
        Ok(Self {
            executor: CommandExecutor::new(allowlist),
            request_counter: AtomicU32::new(1),
            tunnel_dir,
        })
    }

    pub fn handle_msg(&mut self, msg: TunnelMsgOwned, logger: &Logger) -> Option<TunnelMsgOwned> {
        match msg {
            TunnelMsgOwned::CmdRequest { cmd } => {
                let request_id = self.request_counter.fetch_add(1, Ordering::SeqCst);
                let event = CommandEvent::Request {
                    request_id,
                    cmd,
                    scratch_path: None,
                };
                if let Some(command_msg) = self.executor.handle_event(event) {
                    if let CommandMessage::Error { message, .. } = &command_msg {
                        logger.warn(format!("command error: {message}"));
                    }
                    return command_message_to_tunnel(command_msg);
                }
                None
            }
            _ => None,
        }
    }

    pub fn next_outgoing(&mut self) -> Option<TunnelMsgOwned> {
        while let Some(msg) = self.executor.next_outgoing() {
            if let Some(frame) = command_message_to_tunnel(msg) {
                return Some(frame);
            }
        }
        None
    }

    pub fn log_frame_error(&self, detail: &str, raw: &str) {
        let path = self.tunnel_dir.join("errors.log");
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let sanitized = raw.replace('\n', "\\n").replace('\r', "\\r");
            let _ = writeln!(file, "[{now}] {detail}: {sanitized}");
        }
    }
}

fn command_message_to_tunnel(msg: CommandMessage) -> Option<TunnelMsgOwned> {
    match msg {
        CommandMessage::Chunk { stream, data, .. } => match stream {
            CommandStream::Stdout => Some(TunnelMsgOwned::Stdout {
                chunk: data.into_vec(),
            }),
            CommandStream::Stderr => Some(TunnelMsgOwned::Stderr {
                chunk: data.into_vec(),
            }),
        },
        CommandMessage::Exit { code, .. } => Some(TunnelMsgOwned::Exit { code }),
        CommandMessage::Busy { .. } => Some(TunnelMsgOwned::Busy),
        CommandMessage::Error { message, .. } => Some(TunnelMsgOwned::Stderr {
            chunk: message.into_bytes(),
        }),
        CommandMessage::Heartbeat { .. } => Some(TunnelMsgOwned::Heartbeat),
        CommandMessage::Ack { .. } => None,
        CommandMessage::Request { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use crate::app::logger::LogLevel;
    #[cfg(unix)]
    use std::{
        thread,
        time::{Duration, Instant},
    };

    #[cfg(unix)]
    fn wait_for_exit(controller: &mut TunnelController, timeout: Duration) -> TunnelMsgOwned {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(msg) = controller.next_outgoing() {
                if matches!(msg, TunnelMsgOwned::Exit { .. }) {
                    return msg;
                }
            } else if Instant::now() >= deadline {
                panic!("timed out waiting for exit message");
            } else {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn busy_response_blocks_concurrent_commands() {
        let mut controller = TunnelController::new(Vec::new()).unwrap();
        let logger = Logger::new(LogLevel::Info, None).unwrap();

        assert!(controller
            .handle_msg(
                TunnelMsgOwned::CmdRequest {
                    cmd: "sleep 1".into(),
                },
                &logger,
            )
            .is_none());

        let busy = controller
            .handle_msg(TunnelMsgOwned::CmdRequest { cmd: "true".into() }, &logger)
            .expect("expected Busy response");
        assert!(matches!(busy, TunnelMsgOwned::Busy));

        let exit = wait_for_exit(&mut controller, Duration::from_secs(5));
        assert!(matches!(exit, TunnelMsgOwned::Exit { code: 0 }));

        assert!(controller
            .handle_msg(TunnelMsgOwned::CmdRequest { cmd: "true".into() }, &logger,)
            .is_none());

        let final_exit = wait_for_exit(&mut controller, Duration::from_secs(5));
        assert!(matches!(final_exit, TunnelMsgOwned::Exit { code: 0 }));
    }

    #[cfg(unix)]
    #[test]
    fn streams_stdout_chunks_before_exit() {
        let mut controller = TunnelController::new(vec!["echo".into()]).unwrap();
        let logger = Logger::new(LogLevel::Info, None).unwrap();

        assert!(controller
            .handle_msg(
                TunnelMsgOwned::CmdRequest {
                    cmd: "echo hello".into(),
                },
                &logger,
            )
            .is_none());

        let mut stdout = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut exit_code = None;
        while Instant::now() < deadline {
            if let Some(msg) = controller.next_outgoing() {
                match msg {
                    TunnelMsgOwned::Stdout { chunk } => stdout.extend(chunk),
                    TunnelMsgOwned::Exit { code } => {
                        exit_code = Some(code);
                        break;
                    }
                    TunnelMsgOwned::Stderr { chunk } => {
                        panic!("unexpected stderr chunk: {:?}", chunk);
                    }
                    _ => {}
                }
            } else {
                thread::sleep(Duration::from_millis(10));
            }
        }

        assert_eq!(exit_code, Some(0));
        assert!(String::from_utf8_lossy(&stdout).contains("hello"));
    }
}
