use std::collections::VecDeque;

use crc32fast::Hasher;

use crate::{
    payload::{Defaults, RenderFrame, DEFAULT_PAGE_TIMEOUT_MS, DEFAULT_SCROLL_MS},
    Result,
};

/// Maintains a queue of render frames and deduplicates identical payloads.
pub struct RenderState {
    pages: VecDeque<RenderFrame>,
    last_crc: Option<u32>,
    defaults: Defaults,
}

impl RenderState {
    pub fn new(defaults: Option<Defaults>) -> Self {
        Self {
            pages: VecDeque::new(),
            last_crc: None,
            defaults: defaults.unwrap_or(Defaults {
                scroll_speed_ms: DEFAULT_SCROLL_MS,
                page_timeout_ms: DEFAULT_PAGE_TIMEOUT_MS,
            }),
        }
    }

    /// Ingest a JSON frame string. Returns Some(frame) if it is new, None if duplicate.
    pub fn ingest(&mut self, raw: &str) -> Result<Option<RenderFrame>> {
        let crc = checksum_raw(raw);
        if self.last_crc == Some(crc) {
            return Ok(None);
        }
        let frame = RenderFrame::from_payload_json_with_defaults(raw, self.defaults)?;
        self.last_crc = Some(crc);
        self.pages.push_back(frame.clone());
        Ok(Some(frame))
    }

    /// Advance to the next page/frame if available.
    pub fn next_page(&mut self) -> Option<RenderFrame> {
        if self.pages.is_empty() {
            return None;
        }
        let front = self.pages.pop_front();
        if let Some(frame) = front {
            self.pages.push_back(frame.clone());
            Some(frame)
        } else {
            None
        }
    }

    /// Get the current frame without rotating.
    pub fn current(&self) -> Option<&RenderFrame> {
        self.pages.front()
    }

    pub fn len(&self) -> usize {
        self.pages.len()
    }
}

fn checksum_raw(raw: &str) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(raw.as_bytes());
    hasher.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedupes_identical_frames() {
        let mut state = RenderState::new(None);
        let raw = r#"{"line1":"A","line2":"B"}"#;
        let first = state.ingest(raw).unwrap();
        assert!(first.is_some());
        let second = state.ingest(raw).unwrap();
        assert!(second.is_none());
    }

    #[test]
    fn rotates_pages() {
        let mut state = RenderState::new(None);
        state.ingest(r#"{"line1":"A","line2":"B"}"#).unwrap();
        state.ingest(r#"{"line1":"C","line2":"D"}"#).unwrap();
        let first = state.next_page().unwrap();
        assert_eq!(first.line1, "A");
        let second = state.next_page().unwrap();
        assert_eq!(second.line1, "C");
        let third = state.next_page().unwrap();
        assert_eq!(third.line1, "A");
    }
}
