use crate::{Error, Result};
use crc32fast::Hasher;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

pub const TUNNEL_MAX_FRAME_BYTES: usize = 4096;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TunnelMsg<'a> {
    CmdRequest { cmd: Cow<'a, str> },
    Stdout { chunk: Cow<'a, [u8]> },
    Stderr { chunk: Cow<'a, [u8]> },
    Exit { code: i32 },
    Busy,
    Heartbeat,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TunnelMsgOwned {
    CmdRequest { cmd: String },
    Stdout { chunk: Vec<u8> },
    Stderr { chunk: Vec<u8> },
    Exit { code: i32 },
    Busy,
    Heartbeat,
}

impl<'a> TunnelMsg<'a> {
    fn crc32(&self) -> Result<u32> {
        let bytes = serde_json::to_vec(self).map_err(|e| Error::Parse(format!("json: {e}")))?;
        let mut hasher = Hasher::new();
        hasher.update(&bytes);
        Ok(hasher.finalize())
    }

    pub fn into_owned(self) -> TunnelMsgOwned {
        match self {
            TunnelMsg::CmdRequest { cmd } => TunnelMsgOwned::CmdRequest {
                cmd: cmd.into_owned(),
            },
            TunnelMsg::Stdout { chunk } => TunnelMsgOwned::Stdout {
                chunk: chunk.into_owned(),
            },
            TunnelMsg::Stderr { chunk } => TunnelMsgOwned::Stderr {
                chunk: chunk.into_owned(),
            },
            TunnelMsg::Exit { code } => TunnelMsgOwned::Exit { code },
            TunnelMsg::Busy => TunnelMsgOwned::Busy,
            TunnelMsg::Heartbeat => TunnelMsgOwned::Heartbeat,
        }
    }
}

impl TunnelMsgOwned {
    fn crc32(&self) -> Result<u32> {
        let bytes = serde_json::to_vec(self).map_err(|e| Error::Parse(format!("json: {e}")))?;
        let mut hasher = Hasher::new();
        hasher.update(&bytes);
        Ok(hasher.finalize())
    }
}

#[derive(Serialize)]
struct TunnelFrameWriter<'a> {
    msg: &'a TunnelMsgOwned,
    crc32: u32,
}

#[derive(Deserialize)]
struct TunnelFrame<'a> {
    msg: TunnelMsg<'a>,
    crc32: u32,
}

pub fn encode_tunnel_msg(msg: &TunnelMsgOwned) -> Result<String> {
    let crc32 = msg.crc32()?;
    let frame = TunnelFrameWriter { msg, crc32 };
    let json = serde_json::to_string(&frame).map_err(|e| Error::Parse(format!("json: {e}")))?;
    if json.as_bytes().len() > TUNNEL_MAX_FRAME_BYTES {
        return Err(Error::Parse(format!(
            "tunnel frame exceeds {TUNNEL_MAX_FRAME_BYTES} bytes"
        )));
    }
    Ok(json)
}

pub fn decode_tunnel_frame(raw: &str) -> Result<TunnelMsgOwned> {
    if raw.as_bytes().len() > TUNNEL_MAX_FRAME_BYTES {
        return Err(Error::Parse(format!(
            "tunnel frame exceeds {TUNNEL_MAX_FRAME_BYTES} bytes"
        )));
    }
    let frame: TunnelFrame =
        serde_json::from_str(raw).map_err(|e| Error::Parse(format!("json: {e}")))?;
    let computed = frame.msg.crc32()?;
    if computed != frame.crc32 {
        return Err(Error::ChecksumMismatch);
    }
    Ok(frame.msg.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_encoded_cmd_request() {
        let msg = TunnelMsgOwned::CmdRequest {
            cmd: "echo hello".into(),
        };
        let encoded = encode_tunnel_msg(&msg).unwrap();
        let decoded = decode_tunnel_frame(&encoded).unwrap();
        assert_eq!(decoded, msg);
    }

    #[test]
    fn checksum_mismatch_is_detected() {
        let msg = TunnelMsgOwned::CmdRequest {
            cmd: "uptime".into(),
        };
        let encoded = encode_tunnel_msg(&msg).unwrap();
        let mut value: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        if let serde_json::Value::Object(map) = &mut value {
            map.insert(
                "crc32".into(),
                serde_json::Value::Number(serde_json::Number::from(0)),
            );
        }
        let tampered = serde_json::to_string(&value).unwrap();
        let err = decode_tunnel_frame(&tampered).unwrap_err();
        assert!(matches!(err, Error::ChecksumMismatch));
    }

    #[test]
    fn rejects_oversized_frame() {
        let chunk = vec![b'a'; TUNNEL_MAX_FRAME_BYTES];
        let msg = TunnelMsgOwned::Stdout { chunk };
        let err = encode_tunnel_msg(&msg).unwrap_err();
        assert!(format!("{err}").contains("tunnel frame exceeds"));
    }

    #[test]
    fn rejects_oversized_raw_frame() {
        let raw = "{".repeat(TUNNEL_MAX_FRAME_BYTES + 1);
        let err = decode_tunnel_frame(&raw).unwrap_err();
        assert!(format!("{err}").contains("tunnel frame exceeds"));
    }

    #[test]
    fn heartbeat_round_trips_with_crc() {
        let msg = TunnelMsgOwned::Heartbeat;
        let encoded = encode_tunnel_msg(&msg).unwrap();
        let decoded = decode_tunnel_frame(&encoded).unwrap();
        assert_eq!(decoded, msg);
    }
}
