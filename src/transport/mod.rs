use crate::error::Result;
use bytes::BytesMut;

#[async_trait::async_trait]
pub trait DatagramTransport: Send + Sync + 'static {
    async fn recv_from(&self) -> Result<(BytesMut, std::net::SocketAddr)>;
    async fn send_to(&self, buf: &[u8], to: std::net::SocketAddr) -> Result<()>;
    fn max_payload(&self) -> usize; // e.g. 1200 by default
}
