mod icons;
mod parser;
mod schema;

pub use icons::{DisplayMode, Icon};
pub use parser::{Defaults, Payload, RenderFrame};
pub use schema::{
    decode_tunnel_frame, encode_tunnel_msg, TunnelMsg, TunnelMsgOwned, TUNNEL_MAX_FRAME_BYTES,
};

pub const DEFAULT_SCROLL_MS: u64 = 250;
pub const DEFAULT_PAGE_TIMEOUT_MS: u64 = 4000;
