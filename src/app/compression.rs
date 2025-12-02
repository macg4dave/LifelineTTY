use crate::{Error, Result};

/// Compression primitives placeholder for Milestone F (compression support).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionCodec {
    None,
    Lz4,
    Zstd,
}

impl CompressionCodec {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "none" => Some(Self::None),
            "lz4" => Some(Self::Lz4),
            "zstd" => Some(Self::Zstd),
            _ => None,
        }
    }
}

pub fn compress(payload: &[u8], codec: CompressionCodec) -> Result<Vec<u8>> {
    match codec {
        CompressionCodec::None => Ok(payload.to_vec()),
        _ => Err(Error::Parse(
            "compression codec not implemented in skeleton".into(),
        )),
    }
}

pub fn decompress(payload: &[u8], codec: CompressionCodec) -> Result<Vec<u8>> {
    match codec {
        CompressionCodec::None => Ok(payload.to_vec()),
        _ => Err(Error::Parse(
            "decompression codec not implemented in skeleton".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_codec_roundtrips() {
        let buf = b"hello world";
        let c = CompressionCodec::None;
        let compressed = compress(buf, c).unwrap();
        assert_eq!(compressed, buf);
        let decompressed = decompress(&compressed, c).unwrap();
        assert_eq!(decompressed, buf);
    }
}
