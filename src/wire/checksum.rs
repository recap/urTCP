/// RFC 1071 16-bit one's complement checksum.
pub fn ones_complement(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut chunks = data.chunks_exact(2);
    for c in &mut chunks {
        sum += u16::from_be_bytes([c[0], c[1]]) as u32;
    }
    if let Some(&b) = chunks.remainder().first() {
        sum += (b as u32) << 8;
    }
    // fold carries
    while (sum >> 16) != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}
