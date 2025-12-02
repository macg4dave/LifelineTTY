use super::Logger;
use crate::{payload::TunnelMsgOwned, Result, CACHE_DIR};
use std::fs::{create_dir_all, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

const STREAM_CHUNK_SIZE: usize = 512;

#[derive(Debug, Clone, Copy)]
enum StreamKind {
    Stdout,
    Stderr,
}

pub struct TunnelController {
    allowed_commands: Vec<String>,
    outgoing_tx: Sender<TunnelMsgOwned>,
    outgoing_rx: Receiver<TunnelMsgOwned>,
    session_active: bool,
    tunnel_dir: PathBuf,
}

impl TunnelController {
    pub fn new(allowed_commands: Vec<String>) -> Result<Self> {
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
        let (tx, rx) = mpsc::channel();
        Ok(Self {
            allowed_commands,
            outgoing_tx: tx,
            outgoing_rx: rx,
            session_active: false,
            tunnel_dir,
        })
    }

    pub fn handle_msg(&mut self, msg: TunnelMsgOwned, logger: &Logger) -> Option<TunnelMsgOwned> {
        match msg {
            TunnelMsgOwned::CmdRequest { cmd } => {
                if self.session_active {
                    return Some(TunnelMsgOwned::Busy);
                }
                match split_command_line(&cmd) {
                    Ok(tokens) => self.launch_command(tokens, logger),
                    Err(err) => {
                        let reason = format!("command parse error: {err}");
                        let _ = self.queue(TunnelMsgOwned::Stderr {
                            chunk: reason.clone().into_bytes(),
                        });
                        let _ = self.queue(TunnelMsgOwned::Exit { code: 1 });
                        logger.warn(reason);
                        None
                    }
                }
            }
            _ => None,
        }
    }

    pub fn next_outgoing(&mut self) -> Option<TunnelMsgOwned> {
        match self.outgoing_rx.try_recv() {
            Ok(msg) => {
                if matches!(msg, TunnelMsgOwned::Exit { .. }) {
                    self.session_active = false;
                }
                Some(msg)
            }
            Err(_) => None,
        }
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

    fn queue(&self, msg: TunnelMsgOwned) {
        let _ = self.outgoing_tx.send(msg);
    }

    fn launch_command(&mut self, tokens: Vec<String>, logger: &Logger) -> Option<TunnelMsgOwned> {
        if tokens.is_empty() {
            let reason = "empty command".to_string();
            let _ = self.queue(TunnelMsgOwned::Stderr {
                chunk: reason.clone().into_bytes(),
            });
            let _ = self.queue(TunnelMsgOwned::Exit { code: 1 });
            logger.warn(reason);
            return None;
        }
        let program = tokens[0].clone();
        if !self.is_allowed(&program) {
            let reason = format!("command not allowed: {program}");
            let _ = self.queue(TunnelMsgOwned::Stderr {
                chunk: reason.clone().into_bytes(),
            });
            let _ = self.queue(TunnelMsgOwned::Exit { code: 1 });
            logger.warn(reason);
            return None;
        }

        match Command::new(&program)
            .args(&tokens[1..])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                self.session_active = true;
                let tx = self.outgoing_tx.clone();
                if let Some(stdout) = child.stdout.take() {
                    spawn_stream_reader(stdout, StreamKind::Stdout, tx.clone());
                }
                if let Some(stderr) = child.stderr.take() {
                    spawn_stream_reader(stderr, StreamKind::Stderr, tx.clone());
                }
                thread::spawn(move || {
                    let code = match child.wait() {
                        Ok(status) => status.code().unwrap_or(-1),
                        Err(_) => -1,
                    };
                    let _ = tx.send(TunnelMsgOwned::Exit { code });
                });
                None
            }
            Err(err) => {
                let reason = format!("failed to spawn '{program}': {err}");
                let _ = self.queue(TunnelMsgOwned::Stderr {
                    chunk: reason.clone().into_bytes(),
                });
                let _ = self.queue(TunnelMsgOwned::Exit { code: 1 });
                logger.warn(reason);
                None
            }
        }
    }

    fn is_allowed(&self, program: &str) -> bool {
        if self.allowed_commands.is_empty() {
            return true;
        }
        let candidate = std::path::Path::new(program)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(program);
        self.allowed_commands
            .iter()
            .any(|entry| entry == program || entry == candidate)
    }
}

fn spawn_stream_reader<R>(mut reader: R, kind: StreamKind, tx: Sender<TunnelMsgOwned>)
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buf = [0u8; STREAM_CHUNK_SIZE];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = buf[..n].to_vec();
                    let msg = match kind {
                        StreamKind::Stdout => TunnelMsgOwned::Stdout { chunk },
                        StreamKind::Stderr => TunnelMsgOwned::Stderr { chunk },
                    };
                    if tx.send(msg).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
}

fn split_command_line(line: &str) -> std::result::Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut escape = false;
    let trimmed = line.trim();

    if trimmed.is_empty() {
        return Err("empty command".into());
    }

    for ch in trimmed.chars() {
        if escape {
            current.push(ch);
            escape = false;
            continue;
        }
        match ch {
            '\\' => {
                escape = true;
            }
            '\'' | '"' => {
                if let Some(marker) = quote {
                    if marker == ch {
                        quote = None;
                    } else {
                        current.push(ch);
                    }
                } else {
                    quote = Some(ch);
                }
            }
            c if c.is_whitespace() && quote.is_none() => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            c => {
                current.push(c);
            }
        }
    }

    if escape {
        return Err("unterminated escape".into());
    }
    if quote.is_some() {
        return Err("unterminated quote".into());
    }
    if !current.is_empty() {
        args.push(current);
    }
    if args.is_empty() {
        return Err("empty command".into());
    }
    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_command_line_handles_quotes() {
        let cmd = "echo 'hello world'";
        let args = split_command_line(cmd).unwrap();
        assert_eq!(args, vec!["echo", "hello world"]);
    }

    #[test]
    fn split_command_line_detects_unterminated_quote() {
        let err = split_command_line("echo 'foo").unwrap_err();
        assert!(err.contains("unterminated"));
    }

    #[test]
    fn allows_all_commands_when_list_empty() {
        let controller = TunnelController::new(Vec::new()).unwrap();
        assert!(controller.is_allowed("/bin/ls"));
    }

    #[test]
    fn rejects_disallowed_program() {
        let controller = TunnelController::new(vec!["ls".into()]).unwrap();
        assert!(!controller.is_allowed("/bin/echo"));
        assert!(controller.is_allowed("ls"));
        assert!(controller.is_allowed("/bin/ls"));
    }
}
