use crate::Result;
use std::collections::VecDeque;
use std::time::Duration;

#[cfg(test)]
use crate::Error;

pub struct FakeSerialEntry {
    response: Result<String>,
    delay: Option<Duration>,
}

impl FakeSerialEntry {
    pub fn immediate(response: Result<String>) -> Self {
        Self {
            response,
            delay: None,
        }
    }

    pub fn with_delay(response: Result<String>, delay: Duration) -> Self {
        Self {
            response,
            delay: Some(delay),
        }
    }
}

impl From<Result<String>> for FakeSerialEntry {
    fn from(response: Result<String>) -> Self {
        Self::immediate(response)
    }
}

/// Minimal fake serial port used in tests to script reads/writes.
#[derive(Default)]
pub struct FakeSerialPort {
    script: VecDeque<FakeSerialEntry>,
    writes: Vec<String>,
}

impl FakeSerialPort {
    pub fn new(script: Vec<Result<String>>) -> Self {
        Self::with_entries(script.into_iter().map(FakeSerialEntry::from).collect())
    }

    pub fn with_script(script: Vec<FakeSerialEntry>) -> Self {
        Self::with_entries(script)
    }

    fn with_entries(script: Vec<FakeSerialEntry>) -> Self {
        Self {
            script: script.into(),
            writes: Vec::new(),
        }
    }

    pub fn send_command_line(&mut self, line: &str) -> Result<()> {
        self.writes.push(line.to_string());
        Ok(())
    }

    pub fn read_message_line(&mut self, line_buffer: &mut String) -> Result<usize> {
        match self.script.pop_front() {
            Some(entry) => {
                if let Some(delay) = entry.delay {
                    std::thread::sleep(delay);
                }
                match entry.response {
                    Ok(line) => {
                        *line_buffer = line;
                        Ok(line_buffer.len())
                    }
                    Err(err) => Err(err),
                }
            }
            None => Ok(0),
        }
    }

    pub fn writes(&self) -> &[String] {
        &self.writes
    }
}

impl super::LineIo for FakeSerialPort {
    fn send_command_line(&mut self, line: &str) -> crate::Result<()> {
        self.send_command_line(line)
    }

    fn read_message_line(&mut self, buf: &mut String) -> crate::Result<usize> {
        self.read_message_line(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn fake_serial_scripts_reads_and_writes() {
        let mut fake =
            FakeSerialPort::new(vec![Ok("first\n".into()), Err(Error::Parse("boom".into()))]);
        let mut buf = String::new();
        let read = fake.read_message_line(&mut buf).unwrap();
        assert_eq!(read, "first\n".len());
        assert!(fake.read_message_line(&mut buf).is_err());
        fake.send_command_line("PING").unwrap();
        assert_eq!(fake.writes(), &["PING".to_string()]);
    }

    #[test]
    fn scripted_delay_respected() {
        let mut fake = FakeSerialPort::with_script(vec![FakeSerialEntry::with_delay(
            Ok("later".into()),
            Duration::from_millis(5),
        )]);
        let mut buf = String::new();
        let start = Instant::now();
        let read = fake.read_message_line(&mut buf).unwrap();
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(5));
        assert_eq!(buf, "later");
        assert_eq!(read, "later".len());
    }
}
