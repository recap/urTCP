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
