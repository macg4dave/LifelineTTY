use crate::CACHE_DIR;
use std::fs::{self, OpenOptions};
use std::io::Write;

/// Minimal telemetry helper for serial backoff and reconnect counters (P5)
/// Writes small logs into CACHE_DIR.
pub struct Telemetry {
    path: String,
}

impl Telemetry {
    pub fn new(filename: &str) -> Self {
        let path = format!("{}/{}", CACHE_DIR, filename);
        Self { path }
    }

    /// Append a small log line to the telemetry file in CACHE_DIR.
    /// Returns the path used for visibility in tests.
    pub fn append_line(&self, line: &str) -> std::io::Result<String> {
        if let Some(parent) = std::path::Path::new(&self.path).parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(f, "{}", line)?;
        Ok(self.path.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn telemetry_appends_to_file() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("tele.log");
        let tel = Telemetry {
            path: file.to_str().unwrap().to_string(),
        };
        let p = tel.append_line("hello").unwrap();
        assert!(std::path::Path::new(&p).exists());
        let contents = fs::read_to_string(p).unwrap();
        assert!(contents.contains("hello"));
    }
}
