use super::DatagramTransport;
use crate::error::{Result, UtcpError};
use bytes::BytesMut;
use tokio::net::UdpSocket;

pub struct UdpTransport {
    sock: UdpSocket,
    max_payload: usize,
}

impl UdpTransport {
    pub async fn bind<A: tokio::net::ToSocketAddrs>(addr: A, max_payload: usize) -> Result<Self> {
        let sock = UdpSocket::bind(addr)
            .await
            .map_err(|e| UtcpError::Device(e.to_string()))?;
        Ok(Self { sock, max_payload })
    }
    pub async fn connect<A: tokio::net::ToSocketAddrs>(&self, _peer: A) -> Result<()> {
        // Optional: call sock.connect(peer). For multi-peer, skip and always send_to.
        Ok(())
    }
}

#[async_trait::async_trait]
impl DatagramTransport for UdpTransport {
    async fn recv_from(&self) -> Result<(BytesMut, std::net::SocketAddr)> {
        let mut buf = vec![0u8; self.max_payload + 64];
        let (n, from) = self
            .sock
            .recv_from(&mut buf)
            .await
            .map_err(|e| UtcpError::Device(e.to_string()))?;
        buf.truncate(n);
        Ok((BytesMut::from(&buf[..]), from))
    }
    async fn send_to(&self, buf: &[u8], to: std::net::SocketAddr) -> Result<()> {
        self.sock
            .send_to(buf, to)
            .await
            .map_err(|e| UtcpError::Device(e.to_string()))?;
        Ok(())
    }
    fn max_payload(&self) -> usize {
        self.max_payload
    }
}
