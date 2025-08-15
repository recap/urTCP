use crate::error::{Result, UrtcpError};
use bytes::BytesMut;

/// Abstract I/O for the L3 device (reads/writes raw IP frames).
#[async_trait::async_trait]
pub trait NetDevice: Send + Sync + 'static {
    /// Read a single inbound frame into a fresh buffer.
    async fn recv(&self) -> Result<BytesMut>;
    /// Transmit a single outbound frame.
    async fn send(&self, frame: &[u8]) -> Result<()>;
    /// MTU for fragmentation decisions.
    fn mtu(&self) -> usize;
}

/// A simple in-memory loopback device for tests and examples.
pub struct LoopDevice {
    // rx: tokio::sync::mpsc::Receiver<BytesMut>,
    rx: tokio::sync::Mutex<tokio::sync::mpsc::Receiver<BytesMut>>,
    tx: tokio::sync::mpsc::Sender<BytesMut>,
    mtu: usize,
}

impl LoopDevice {
    pub fn pair(mtu: usize) -> (Self, Self) {
        let (a_tx, a_rx) = tokio::sync::mpsc::channel(1024);
        let (b_tx, b_rx) = tokio::sync::mpsc::channel(1024);
        (
            Self {
                rx: tokio::sync::Mutex::new(a_rx),
                tx: b_tx.clone(),
                mtu,
            },
            Self {
                rx: tokio::sync::Mutex::new(b_rx),
                tx: a_tx.clone(),
                mtu,
            },
        )
    }
}

#[async_trait::async_trait]
impl NetDevice for LoopDevice {
    // async fn recv(&self) -> Result<BytesMut> {
    //     self.rx
    //         .recv()
    //         .await
    //         .ok_or_else(|| UrtcpError::Device("rx closed".into()))
    // }
    async fn recv(&self) -> Result<BytesMut> {
        let mut rx = self.rx.lock().await;
        rx.recv()
            .await
            .ok_or_else(|| UrtcpError::Device("rx closed".into()))
    }
    async fn send(&self, frame: &[u8]) -> Result<()> {
        self.tx
            .send(BytesMut::from(frame))
            .await
            .map_err(|e| UrtcpError::Device(e.to_string()))
    }
    fn mtu(&self) -> usize {
        self.mtu
    }
}

#[cfg(feature = "tun-backend")]
pub mod tun_backend {
    use super::*;
    use tun::{AbstractDevice, Configuration, Device as TunDev};

    /// Minimal TUN wrapper (IPv4 only, for now).
    pub struct TunDevice {
        dev: TunDev,
        mtu: usize,
        ifname: String,
    }
    impl TunDevice {
        pub fn new(name: &str, mtu: usize) -> Result<Self> {
            let mut cfg = Configuration::default();
            // cfg.up();
            cfg.address((10, 0, 0, 1))
                .netmask((255, 255, 255, 0))
                .mtu(mtu as u16)
                .up();

            let mut dev = tun::create(&cfg).map_err(|e| UrtcpError::Device(e.to_string()))?;
            let ifname = dev
                .tun_name()
                .map_err(|e| UrtcpError::Device(e.to_string()))?;
            Ok(Self { dev, mtu, ifname })
        }

        pub fn ifname(&self) -> &str {
            &self.ifname
        }
    }
    #[async_trait::async_trait]
    impl NetDevice for TunDevice {
        async fn recv(&self) -> Result<bytes::BytesMut> {
            // NOTE: TunDev is blocking; in production wrap with spawn_blocking or use async fd
            use tokio::task;
            let mut buf = vec![0u8; self.mtu + 64];
            let n = task::spawn_blocking(move || {
                // SAFETY: demo
                // Read directly; replace with async-io wrapper in real code
                use std::io::Read;
                // This is placeholder pseudo; adapt to tun crate's API
                Ok::<usize, std::io::Error>(0)
            })
            .await
            .map_err(|e| UrtcpError::Device(format!("join error: {e}")))?
            .map_err(|e| UrtcpError::Device(e.to_string()))?;
            buf.truncate(n);
            Ok(bytes::BytesMut::from(&buf[..]))
        }
        async fn send(&self, _frame: &[u8]) -> Result<()> {
            todo!("write to tun device")
        }
        fn mtu(&self) -> usize {
            self.mtu
        }
    }
}
