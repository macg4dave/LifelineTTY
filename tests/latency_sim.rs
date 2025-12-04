use lifelinetty::payload::{
    decode_tunnel_frame, encode_tunnel_msg, Defaults as PayloadDefaults, TunnelMsgOwned,
    DEFAULT_PAGE_TIMEOUT_MS, DEFAULT_SCROLL_MS,
};
use lifelinetty::serial::fake::{FakeSerialEntry, FakeSerialPort};
use lifelinetty::state::RenderState;
use std::time::{Duration, Instant};

/// Ensures the render loop keeps accepting LCD frames even when tunnel traffic arrives with delays.
#[test]
fn tunnel_latency_allows_lcd_frames() {
    let mut serial = FakeSerialPort::with_script(vec![
        FakeSerialEntry::immediate(Ok(
            r#"{"schema_version":1,"line1":"FRAME 1","line2":"","scroll":false}"#.into(),
        )),
        FakeSerialEntry::with_delay(
            Ok(encode_tunnel_msg(&TunnelMsgOwned::CmdRequest { cmd: "ls".into() }).unwrap()),
            Duration::from_millis(8),
        ),
        FakeSerialEntry::immediate(Ok(
            r#"{"schema_version":1,"line1":"FRAME 2","line2":"","scroll":false}"#.into(),
        )),
    ]);

    let mut state = RenderState::new(Some(PayloadDefaults {
        scroll_speed_ms: DEFAULT_SCROLL_MS,
        page_timeout_ms: DEFAULT_PAGE_TIMEOUT_MS,
    }));

    let mut frames = Vec::new();
    let mut buffer = String::new();
    let start = Instant::now();

    while serial.read_message_line(&mut buffer).unwrap() > 0 {
        let trimmed = buffer.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.contains("\"msg\"") && trimmed.contains("\"crc32\"") {
            let decoded = decode_tunnel_frame(trimmed).expect("valid tunnel frame");
            assert!(matches!(decoded, TunnelMsgOwned::CmdRequest { .. }));
            continue;
        }
        if let Some(frame) = state.ingest(trimmed).unwrap() {
            frames.push(frame);
        }
    }

    let elapsed = start.elapsed();
    assert_eq!(frames.len(), 2);
    assert!(elapsed >= Duration::from_millis(8));
}
