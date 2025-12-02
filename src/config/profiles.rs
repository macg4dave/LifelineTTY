use crate::{Error, Result};
use std::collections::HashMap;

/// Config skeleton for polling profiles (P18)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PollingProfiles {
    /// map of metric name to interval_ms
    pub profiles: HashMap<String, u64>,
}

impl Default for PollingProfiles {
    fn default() -> Self {
        Self {
            profiles: HashMap::new(),
        }
    }
}

impl PollingProfiles {
    pub fn parse_from_toml(raw: &str) -> Result<Self> {
        // Minimal parser: expect lines like name = 1000
        let mut map = HashMap::new();
        for (idx, line) in raw.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = trimmed.split_once('=') {
                let k = k.trim().trim_matches('"');
                let v = v.trim();
                let val: u64 = v.parse().map_err(|_| {
                    Error::InvalidArgs(format!("invalid profile interval on line {}", idx + 1))
                })?;
                map.insert(k.to_string(), val);
            } else {
                return Err(Error::InvalidArgs(format!(
                    "invalid profile literal on line {}",
                    idx + 1
                )));
            }
        }
        Ok(Self { profiles: map })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_profiles_simple() {
        let raw = r#"
            cpu = 1000
            mem = 5000
        "#;
        let p = PollingProfiles::parse_from_toml(raw).unwrap();
        assert_eq!(p.profiles.get("cpu"), Some(&1000u64));
        assert_eq!(p.profiles.get("mem"), Some(&5000u64));
    }
}
