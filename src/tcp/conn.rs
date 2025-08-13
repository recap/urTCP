use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};

use crate::error::*;
use crate::wire::tcp::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Quad {
    pub src_ip: [u8; 4],
    pub src_port: u16,
    pub dst_ip: [u8; 4],
    pub dst_port: u16,
}

#[derive(Debug, Clone, Copy)]
pub enum State {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    LastAck,
    TimeWait,
}

pub struct Connection {
    pub id: Quad,
    pub state: State,
    pub iss: u32,
    pub snd_una: u32,
    pub snd_nxt: u32,
    pub rcv_nxt: u32,
    pub cwnd: usize,
    pub ssthresh: usize,
    pub rto: Duration,
    pub last_activity: Instant,
    // TX/RX queues (simplified)
    pub app_rx: mpsc::Sender<Vec<u8>>,
    pub app_tx: mpsc::Receiver<Vec<u8>>,
}

impl Connection {
    pub fn new(
        id: Quad,
        state: State,
        app_rx: mpsc::Sender<Vec<u8>>,
        app_tx: mpsc::Receiver<Vec<u8>>,
    ) -> Self {
        Self {
            id,
            state,
            iss: 0,
            snd_una: 0,
            snd_nxt: 0,
            rcv_nxt: 0,
            cwnd: 1_460,
            ssthresh: 65_535,
            rto: Duration::from_millis(300),
            last_activity: Instant::now(),
            app_rx,
            app_tx,
        }
    }

    /// Handle an inbound TCP segment (without IP header).
    pub fn on_segment(&mut self, _seg: &[u8]) -> Result<()> {
        // parse header, update state machine, ack, push payload to app_rx
        Err(UtcpError::NotImplemented("conn.on_segment"))
    }

    /// Pull application data to send, make segments
    pub fn poll_send(&mut self) -> Result<Option<Vec<u8>>> {
        // Nagle, cwnd/flight control, build segments, advance snd_nxt
        Ok(None)
    }

    /// Called by timer wheel on RTO
    pub fn on_retransmit_timeout(&mut self) -> Result<()> {
        // backoff, retransmit snd_una..snd_nxt
        Err(UtcpError::NotImplemented("RTO"))
    }
}

/// Commands from sockets to the stack’s TCP engine.
pub enum TcpCmd {
    Connect(Quad, oneshot::Sender<Result<()>>),
    Listen(u16, oneshot::Sender<Result<()>>),
    Send(Quad, Vec<u8>),
    Close(Quad),
}
