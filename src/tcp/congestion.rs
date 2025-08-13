#[derive(Debug, Clone)]
pub struct Reno {
    pub cwnd: usize,
    pub ssthresh: usize,
}
impl Default for Reno {
    fn default() -> Self {
        Self {
            cwnd: 1460,
            ssthresh: 65_535,
        }
    }
}
