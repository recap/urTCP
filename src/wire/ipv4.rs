use bytes::{BufMut, BytesMut};

#[derive(Clone, Copy, Debug)]
pub struct Ipv4Addr(pub [u8; 4]);

#[derive(Clone, Debug)]
pub struct Ipv4Header {
    pub src: Ipv4Addr,
    pub dst: Ipv4Addr,
    pub proto: u8, // 6 for TCP
    pub ident: u16,
    pub ttl: u8,
}

impl Ipv4Header {
    pub fn encode(&self, payload: &[u8]) -> BytesMut {
        let ihl = 5u8;
        let ver_ihl = (4u8 << 4) | ihl;
        let total_len = (ihl as usize * 4 + payload.len()) as u16;

        let mut buf = BytesMut::with_capacity(total_len as usize);
        buf.put_u8(ver_ihl);
        buf.put_u8(0); // DSCP/ECN
        buf.put_u16(total_len);
        buf.put_u16(self.ident);
        buf.put_u16(0x4000); // flags/frag: DF
        buf.put_u8(self.ttl);
        buf.put_u8(self.proto);
        buf.put_u16(0); // checksum placeholder
        buf.extend_from_slice(&self.src.0);
        buf.extend_from_slice(&self.dst.0);
        // compute header checksum (header only)
        let cksum = super::checksum::ones_complement(&buf[..ihl as usize * 4]);
        buf[10] = (cksum >> 8) as u8;
        buf[11] = (cksum & 0xff) as u8;
        buf.extend_from_slice(payload);
        buf
    }
}

pub struct Ipv4View<'a> {
    pub src: [u8; 4],
    pub dst: [u8; 4],
    pub proto: u8,
    pub ihl_bytes: usize,
    pub payload: &'a [u8],
}

pub fn parse_ipv4(frame: &[u8]) -> Option<Ipv4View<'_>> {
    if frame.len() < 20 {
        return None;
    }
    let ver_ihl = frame[0];
    if ver_ihl >> 4 != 4 {
        return None;
    }
    let ihl = (ver_ihl & 0x0f) as usize;
    let ihl_bytes = ihl * 4;
    if ihl_bytes < 20 || frame.len() < ihl_bytes {
        return None;
    }
    let total_len = u16::from_be_bytes([frame[2], frame[3]]) as usize;
    if total_len > frame.len() {
        return None;
    }
    let proto = frame[9];
    let src = [frame[12], frame[13], frame[14], frame[15]];
    let dst = [frame[16], frame[17], frame[18], frame[19]];
    let payload = &frame[ihl_bytes..total_len];
    Some(Ipv4View {
        src,
        dst,
        proto,
        ihl_bytes,
        payload,
    })
}
