//! Manifest+resume stub for the transfer module. Will eventually record chunk ids, checksums, and offsets in RAM-disk manifests.

/// Placeholder manifest entry describing stored metadata.
pub struct ManifestEntry;

impl ManifestEntry {
    /// Placeholder method to compute where the transfer left off.
    pub fn resume_point(&self) -> u64 {
        0
    }
}
