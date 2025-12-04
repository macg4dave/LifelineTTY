use lifelinetty::{
    lcd::Lcd,
    payload::{
        decode_command_frame, encode_command_frame, CommandMessage, Defaults,
        DEFAULT_PAGE_TIMEOUT_MS, DEFAULT_SCROLL_MS,
    },
    state::RenderState,
    Error,
};
use serde_json::Value;

#[test]
fn integration_parses_and_states() {
    let mut state = RenderState::new(Some(Defaults {
        scroll_speed_ms: DEFAULT_SCROLL_MS,
        page_timeout_ms: DEFAULT_PAGE_TIMEOUT_MS,
    }));
    let raw = r#"{"schema_version":1,"line1":"CPU","line2":"42%","bar":42,"scroll":false}"#;
    let frame = state.ingest(raw).unwrap().unwrap();
    assert_eq!(frame.bar_percent, Some(42));
    assert!(!frame.scroll_enabled);
    assert_eq!(state.len(), 1);
}

#[test]
#[ignore]
fn smoke_lcd_write_lines_stub() {
    let mut lcd = Lcd::new(
        16,
        2,
        lifelinetty::config::DEFAULT_PCF8574_ADDR,
        lifelinetty::config::DEFAULT_DISPLAY_DRIVER,
    )
    .unwrap();
    lcd.write_lines("HELLO", "WORLD").unwrap();
}

#[test]
fn command_frame_detects_bad_crc() {
    let msg = CommandMessage::Request {
        request_id: 1,
        cmd: "echo hi".into(),
        scratch_path: None,
    };
    let encoded = encode_command_frame(&msg).expect("encode frame");
    let mut value: Value = serde_json::from_str(&encoded).expect("deserialize frame");
    if let Value::Object(map) = &mut value {
        map.insert("crc32".into(), Value::from(0));
    }
    let tampered = serde_json::to_string(&value).expect("serialize tampered");
    let err = decode_command_frame(&tampered).unwrap_err();
    assert!(matches!(err, Error::ChecksumMismatch));
}
