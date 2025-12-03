//! Tunnel framing placeholder for the command tunnel. This module will eventually serialize and
//! deserialize newline-delimited frames containing commands, stdout/stderr chunks, error
//! codes, and CRC metadata.

/// Represents a parsed tunnel frame payload that will be validated once the parser is wired up.
pub struct TunnelFrame;

impl TunnelFrame {
    /// Placeholder constructor describing how frames will be built once the CRC helpers land.
    pub fn new() -> Self {
        Self
    }
}
