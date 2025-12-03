//! Transfer pipeline stub for the remote file transport. Once wired, this module will buffer chunks, respect
//! CRCs, and read/write to `/run/serial_lcd_cache` during transfers.

/// Placeholder representing a chunk sender or receiver.
pub struct ChunkTransfer;

impl ChunkTransfer {
    /// Placeholder start method describing how a transfer will be initiated.
    pub fn start(&self) {
        // TODO: integrate buffered IO and resume manifests here.
    }
}
