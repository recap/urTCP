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

// impl Connection {
//     pub fn new(
//         id: Quad,
//         state: State,
//         app_rx: mpsc::Sender<Vec<u8>>,
//         app_tx: mpsc::Receiver<Vec<u8>>,
//     ) -> Self {
//         Self {
//             id,
//             state,
//             iss: 0,
//             snd_una: 0,
//             snd_nxt: 0,
//             rcv_nxt: 0,
//             cwnd: 1_460,
//             ssthresh: 65_535,
//             rto: Duration::from_millis(300),
//             last_activity: Instant::now(),
//             app_rx,
//             app_tx,
//         }
//     }
//
//     /// Handle an inbound TCP segment (without IP header).
//     pub fn on_segment(&mut self, _seg: &[u8]) -> Result<()> {
//         // parse header, update state machine, ack, push payload to app_rx
//         Err(UrtcpError::NotImplemented("conn.on_segment"))
//     }
//
//     /// Pull application data to send, make segments
//     pub fn poll_send(&mut self) -> Result<Option<Vec<u8>>> {
//         // Nagle, cwnd/flight control, build segments, advance snd_nxt
//         Ok(None)
//     }
//
//     /// Called by timer wheel on RTO
//     pub fn on_retransmit_timeout(&mut self) -> Result<()> {
//         // backoff, retransmit snd_una..snd_nxt
//         Err(UrtcpError::NotImplemented("RTO"))
//     }
// }

use crate::wire::tcp::{self, FLAG_ACK, FLAG_FIN, FLAG_SYN};

pub enum RxAction {
    None,
    SendAck,
    SendSynAck,
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

    /// Called by timer wheel on RTO
    pub fn on_retransmit_timeout(&mut self) -> Result<()> {
        // backoff, retransmit snd_una..snd_nxt
        Err(UrtcpError::NotImplemented("RTO"))
    }

    pub fn on_segment(&mut self, seg: &[u8]) -> Result<RxAction> {
        let tv = tcp::parse(seg).ok_or(UrtcpError::Malformed)?;
        match self.state {
            State::SynSent => {
                let got_syn = (tv.flags & FLAG_SYN) != 0;
                let got_ack = (tv.flags & FLAG_ACK) != 0;
                if got_syn && got_ack && tv.ack == self.iss.wrapping_add(1) {
                    self.rcv_nxt = tv.seq.wrapping_add(1);
                    self.snd_una = tv.ack;
                    self.snd_nxt = self.snd_una;
                    self.state = State::Established;
                    return Ok(RxAction::SendAck);
                }
            }
            State::SynReceived => {
                if (tv.flags & FLAG_ACK) != 0 && tv.ack == self.iss.wrapping_add(1) {
                    self.snd_una = tv.ack;
                    self.state = State::Established;
                    return Ok(RxAction::None);
                }
            }
            State::Established => {
                if (tv.flags & FLAG_FIN) != 0 {
                    self.rcv_nxt = tv.seq.wrapping_add(1);
                    // next: transition to CloseWait/LastAck etc.
                    return Ok(RxAction::SendAck);
                }
                if !tv.payload.is_empty() {
                    let _ = self.app_rx.try_send(tv.payload.to_vec());
                    self.rcv_nxt = tv.seq.wrapping_add(tv.payload.len() as u32);
                    return Ok(RxAction::SendAck);
                }
            }
            _ => {}
        }
        Ok(RxAction::None)
    }

    pub fn poll_send(&mut self) -> Result<Option<Vec<u8>>> {
        if let Ok(Some(buf)) = self.app_tx.try_recv().map(Some).or(Ok(None)) {
            self.snd_nxt = self.snd_nxt.wrapping_add(buf.len() as u32);
            return Ok(Some(buf));
        }
        Ok(None)
    }
}

/// Commands from sockets to the stack’s TCP engine.
pub enum TcpCmd {
    Connect(Quad, oneshot::Sender<Result<()>>),
    Listen(u16, oneshot::Sender<Result<()>>),
    Send(Quad, Vec<u8>),
    Close(Quad),
}
