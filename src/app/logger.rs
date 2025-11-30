use std::io::Write;

/// Simple stderr/file logger used across the app module.
pub struct Logger {
    file: Option<std::fs::File>,
}

impl Logger {
    pub fn new() -> Self {
        let path = std::env::var("SERIALLCD_LOG_PATH").ok();
        let file = path.and_then(|p| {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(p)
                .ok()
        });
        Self { file }
    }

    pub fn log(&self, msg: String) {
        eprintln!("{msg}");
        if let Some(file) = self.file.as_ref() {
            if let Ok(mut clone) = file.try_clone() {
                let _ = writeln!(clone, "{msg}");
            }
        }
    }
}
