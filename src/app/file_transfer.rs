use crate::{Error, Result};
use std::path::Path;

/// File transfer manager placeholder for Milestone C.
/// This module provides a minimal in-repo API to build tests and further
/// implementation around chunking, checksums and resume functionality.
pub struct FileTransferManager {
    /// path to a cache directory (should be CACHE_DIR in production)
    pub cache_dir: String,
}

impl FileTransferManager {
    pub fn new(cache_dir: &str) -> Self {
        Self {
            cache_dir: cache_dir.to_string(),
        }
    }

    /// Prepare sending a local file by validating it exists and returning a
    /// transfer id (stubbed in this skeleton).
    pub fn prepare_send(&self, path: &str) -> Result<String> {
        if !Path::new(path).exists() {
            return Err(Error::Parse(format!("file not found: {path}")));
        }
        // Real implementation will stage the file into cache and create a resume manifest
        Ok("transfer-id-stub".to_string())
    }

    /// Accept a chunk into the receive pipeline (stubbed). Real code will
    /// validate chunk id, crc and append to a temporary file in the cache dir.
    pub fn receive_chunk(
        &self,
        _transfer_id: &str,
        _chunk_idx: u64,
        _payload: &[u8],
    ) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn prepare_send_rejects_missing_file() {
        let m = FileTransferManager::new("/tmp");
        let err = m.prepare_send("/path/does/not/exist").unwrap_err();
        assert!(format!("{err}").contains("file not found"));
    }

    #[test]
    fn prepare_send_accepts_existing_file() {
        let dir = tempdir().unwrap();
        let fpath = dir.path().join("f.txt");
        File::create(&fpath).unwrap();
        let m = FileTransferManager::new(dir.path().to_str().unwrap());
        let id = m.prepare_send(fpath.to_str().unwrap()).unwrap();
        assert_eq!(id, "transfer-id-stub");
    }
}
