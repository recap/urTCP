use tokio::sync::{mpsc, oneshot};
use tokio::time::{Duration, timeout};

use super::conn::{Quad, TcpCmd};
use crate::error::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TcpSocketAddr {
    pub ip: [u8; 4],
    pub port: u16,
}

/// A minimal async stream-like API backed by the stack.
pub struct TcpStream {
    id: Quad,
    tx_cmd: mpsc::Sender<TcpCmd>,
    app_rx: mpsc::Receiver<Vec<u8>>,
    app_tx: mpsc::Sender<Vec<u8>>,
}

pub struct TcpListener {
    local: TcpSocketAddr,
    tx_cmd: mpsc::Sender<TcpCmd>,
    // For skeleton: not fully implemented
}

impl TcpStream {
    pub async fn connect(
        tx_cmd: mpsc::Sender<TcpCmd>,
        local: TcpSocketAddr,
        remote: TcpSocketAddr,
    ) -> Result<Self> {
        let (app_tx_s, app_tx_r) = mpsc::channel(64);
        let (app_rx_s, app_rx_r) = mpsc::channel(64);

        let id = Quad {
            src_ip: local.ip,
            src_port: local.port,
            dst_ip: remote.ip,
            dst_port: remote.port,
        };

        let (reply_tx, reply_rx) = oneshot::channel();
        tx_cmd
            .send(TcpCmd::Connect(id, reply_tx))
            .await
            .map_err(|_| UrtcpError::Device("control channel".into()))?;
        timeout(Duration::from_secs(2), reply_rx)
            .await
            .map_err(|_| UrtcpError::Device("connect timeout".into()))?
            .map_err(|_| UrtcpError::Device("connect drop".into()))??;

        Ok(Self {
            id,
            tx_cmd,
            app_rx: app_rx_r,
            app_tx: app_tx_s,
        })
    }

    pub async fn write_all(&self, data: Vec<u8>) -> Result<()> {
        self.tx_cmd
            .send(TcpCmd::Send(self.id, data))
            .await
            .map_err(|_| UrtcpError::Device("control channel".into()))
    }

    pub async fn read(&mut self) -> Result<Option<Vec<u8>>> {
        Ok(self.app_rx.recv().await)
    }

    pub async fn close(&self) -> Result<()> {
        self.tx_cmd
            .send(TcpCmd::Close(self.id))
            .await
            .map_err(|_| UrtcpError::Device("control channel".into()))
    }
}

impl TcpListener {
    pub async fn bind(tx_cmd: mpsc::Sender<TcpCmd>, local: TcpSocketAddr) -> Result<Self> {
        let (reply_tx, reply_rx) = oneshot::channel();
        tx_cmd
            .send(TcpCmd::Listen(local.port, reply_tx))
            .await
            .map_err(|_| UrtcpError::Device("control channel".into()))?;
        reply_rx
            .await
            .map_err(|_| UrtcpError::Device("listen drop".into()))??;
        Ok(Self { local, tx_cmd })
    }

    // In a full impl, accept() would wait on a channel for new Quad + app queues.
}
