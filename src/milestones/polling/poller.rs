//! Poller stub for the live metric collector. Will gather CPU, memory, temperature, and disk metrics without blocking serial ingest.

/// Placeholder metric struct to be populated by future /proc readers.
pub struct MetricsSnapshot;

impl MetricsSnapshot {
    /// Placeholder builder that will later sample `/proc` and `/sys` data.
    pub fn sample() -> Self {
        MetricsSnapshot
    }
}
