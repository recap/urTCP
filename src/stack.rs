use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::device::NetDevice;
use crate::error::*;
use crate::tcp::{
    conn::{Connection, Quad, State, TcpCmd},
    timers::TimerWheel,
};
use crate::wire::{ipv4::Ipv4Header, tcp::TcpHeader};

#[derive(Clone, Debug)]
pub struct StackConfig {
    pub local_ip: [u8; 4],
    pub ttl: u8,
    pub ident_seed: u16,
    pub mtu: usize,
}

pub struct Stack<D: NetDevice> {
    dev: D,
    cfg: StackConfig,
    conns: HashMap<Quad, Connection>,
    listeners: HashMap<u16, ()>, // port -> placeholder
    tx_cmd: mpsc::Sender<TcpCmd>,
    rx_cmd: mpsc::Receiver<TcpCmd>,
}

impl<D: NetDevice> Stack<D> {
    pub fn new(dev: D, cfg: StackConfig) -> Self {
        let (tx_cmd, rx_cmd) = mpsc::channel(1024);
        Self {
            dev,
            cfg,
            conns: HashMap::new(),
            listeners: HashMap::new(),
            tx_cmd,
            rx_cmd,
        }
    }

    pub fn control(&self) -> mpsc::Sender<TcpCmd> {
        self.tx_cmd.clone()
    }

    /// Main event loop: device frames, control cmds, and timers.
    pub async fn run(mut self) -> Result<()> {
        let mut timers = TimerWheel::new();
        loop {
            tokio::select! {
                // Inbound frame from device
                frame = self.dev.recv() => {
                    let frame = frame?;
                    self.on_frame(&frame)?;
                }
                // Control plane (connect/listen/send/close)
                Some(cmd) = self.rx_cmd.recv() => {
                    self.on_cmd(cmd).await?;
                }
                // Timer ticks
                _ = timers.tick() => {
                    self.on_tick()?;
                }
            }
        }
    }

    fn on_frame(&mut self, frame: &[u8]) -> Result<()> {
        // Parse IPv4 + TCP, route to connection
        // NOTE: skeleton only
        let _ = frame;
        Err(UrtcpError::NotImplemented("parse ipv4/tcp"))
    }

    async fn on_cmd(&mut self, cmd: TcpCmd) -> Result<()> {
        match cmd {
            TcpCmd::Connect(id, reply) => {
                // Create connection in SynSent, send SYN
                let (app_rx_s, _app_rx_r) = tokio::sync::mpsc::channel(64);
                let (_app_tx_s, app_tx_r) = tokio::sync::mpsc::channel(64);
                let mut conn = Connection::new(id, State::SynSent, app_rx_s, app_tx_r);
                // Build SYN
                let tcp = TcpHeader {
                    src_port: id.src_port,
                    dst_port: id.dst_port,
                    seq: conn.iss,
                    ack: 0,
                    data_offset: 5,
                    flags: crate::wire::tcp::FLAG_SYN,
                    window: 65535,
                    urg_ptr: 0,
                    options: bytes::BytesMut::new(),
                };
                let seg = tcp.encode(&[], id.src_ip, id.dst_ip);
                let ip = Ipv4Header {
                    src: crate::wire::ipv4::Ipv4Addr(id.src_ip),
                    dst: crate::wire::ipv4::Ipv4Addr(id.dst_ip),
                    proto: 6,
                    ident: self.cfg.ident_seed,
                    ttl: self.cfg.ttl,
                }
                .encode(&seg);
                self.dev.send(&ip).await?;
                self.conns.insert(id, conn);
                let _ = reply.send(Ok(()));
            }
            TcpCmd::Listen(port, reply) => {
                self.listeners.insert(port, ());
                let _ = reply.send(Ok(()));
            }
            TcpCmd::Send(id, _data) => {
                if let Some(conn) = self.conns.get_mut(&id) {
                    let _ = conn.poll_send()?; // build segments (todo)
                } else {
                    return Err(UrtcpError::ConnNotFound);
                }
            }
            TcpCmd::Close(id) => {
                // Transition to FIN-WAIT (active) or LAST-ACK (passive)
                let _ = id;
            }
        }
        Ok(())
    }

    fn on_tick(&mut self) -> Result<()> {
        // Drive retransmissions, delayed ACKs, persist timer, TIME-WAIT
        for (_k, _c) in self.conns.iter_mut() {
            // _c.on_retransmit_timeout()?;
        }
        Ok(())
    }
}
