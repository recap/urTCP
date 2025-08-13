use bytes::{BufMut, BytesMut};

#[derive(Clone, Debug, Default)]
pub struct TcpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub seq: u32,
    pub ack: u32,
    pub data_offset: u8, // 5..15 (32-bit words)
    pub flags: u16,      // NS|CWR|ECE|URG|ACK|PSH|RST|SYN|FIN
    pub window: u16,
    pub urg_ptr: u16,
    pub options: BytesMut,
}

pub const FLAG_FIN: u16 = 0x01;
pub const FLAG_SYN: u16 = 0x02;
pub const FLAG_RST: u16 = 0x04;
pub const FLAG_PSH: u16 = 0x08;
pub const FLAG_ACK: u16 = 0x10;

impl TcpHeader {
    pub fn encode(&self, payload: &[u8], src_ip: [u8; 4], dst_ip: [u8; 4]) -> BytesMut {
        let hdr_len = (self.data_offset as usize) * 4;
        let total_len = hdr_len + payload.len();
        let mut buf = BytesMut::with_capacity(total_len);
        buf.put_u16(self.src_port);
        buf.put_u16(self.dst_port);
        buf.put_u32(self.seq);
        buf.put_u32(self.ack);
        let off_res_flags = ((self.data_offset as u16) << 12) | (self.flags & 0x01ff);
        buf.put_u16(off_res_flags);
        buf.put_u16(self.window);
        buf.put_u16(0); // checksum placeholder
        buf.put_u16(self.urg_ptr);
        if !self.options.is_empty() {
            buf.extend_from_slice(&self.options);
        }
        buf.extend_from_slice(payload);

        // pseudo-header checksum
        let mut pseudo = BytesMut::with_capacity(12 + total_len);
        pseudo.extend_from_slice(&src_ip);
        pseudo.extend_from_slice(&dst_ip);
        pseudo.put_u8(0);
        pseudo.put_u8(6);
        pseudo.put_u16(total_len as u16);
        pseudo.extend_from_slice(&buf);

        let cksum = super::checksum::ones_complement(&pseudo);
        buf[16] = (cksum >> 8) as u8;
        buf[17] = (cksum & 0xff) as u8;
        buf
    }
}
