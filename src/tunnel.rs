use crate::error::{Result, UtcpError};
use bytes::{Buf, BufMut, BytesMut};

pub const MAGIC: [u8; 4] = *b"uTCP";
pub const VERSION: u8 = 1;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Hello = 0,
    Data = 1,
    Ping = 2,
    Pong = 3,
}

#[derive(Clone, Debug)]
pub struct TunnelHdr {
    pub cid: u32, // connection id (local to receiver)
    pub kind: Kind,
}

impl TunnelHdr {
    pub fn encode(&self, payload: &[u8]) -> BytesMut {
        let mut b = BytesMut::with_capacity(4 + 1 + 4 + 1 + 2 + payload.len());
        b.extend_from_slice(&MAGIC); // 4
        b.put_u8(VERSION); // 1
        b.put_u32(self.cid); // 4
        b.put_u8(self.kind as u8); // 1
        b.put_u16(payload.len() as u16); // 2
        b.extend_from_slice(payload);
        b
    }
    pub fn decode(mut buf: BytesMut) -> Result<(Self, BytesMut)> {
        if buf.len() < 12 {
            return Err(UtcpError::Malformed);
        }
        if &buf[..4] != MAGIC {
            return Err(UtcpError::Malformed);
        }
        let _ = buf.split_to(4);
        let ver = buf.get_u8();
        if ver != VERSION {
            return Err(UtcpError::Malformed);
        }
        let cid = buf.get_u32();
        let kind = match buf.get_u8() {
            0 => Kind::Hello,
            1 => Kind::Data,
            2 => Kind::Ping,
            3 => Kind::Pong,
            _ => return Err(UtcpError::Malformed),
        };
        let len = buf.get_u16() as usize;
        if buf.len() < len {
            return Err(UtcpError::Malformed);
        }
        let payload = buf.split_to(len);
        Ok((Self { cid, kind }, payload))
    }
}
