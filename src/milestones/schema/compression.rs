//! Compression stub for schema validation. This module will eventually wrap LZ4/zstd helpers ensuring buffers stay below 1 MB.

/// Placeholder compression codec descriptor.
pub enum CompressionCodec {
    Lz4,
    Zstd,
}

impl CompressionCodec {
    /// Placeholder description of the codec that will be negotiated later.
    pub fn description(&self) -> &'static str {
        match self {
            CompressionCodec::Lz4 => "lz4",
            CompressionCodec::Zstd => "zstd",
        }
    }
}
