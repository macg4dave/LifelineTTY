mod icons;
mod parser;

pub use icons::{DisplayMode, Icon};
pub use parser::{Defaults, Payload, RenderFrame};

pub const DEFAULT_SCROLL_MS: u64 = 250;
pub const DEFAULT_PAGE_TIMEOUT_MS: u64 = 4000;
