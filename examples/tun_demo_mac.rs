// examples/tun_demo_mac.rs
use urtcp::device::tun_backend::TunDevice;
use urtcp::tcp::socket::{TcpSocketAddr, TcpStream};
use urtcp::{Stack, StackConfig};

use std::process::Command;
use tokio::time::{Duration, sleep};
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

// If your TunDevice doesn't expose the name, add this in tun_backend:
// impl TunDevice { pub fn ifname(&self) -> &str { self.dev.name() } }
// trait IfName {
//     fn ifname(&self) -> &str;
// }
// impl IfName for TunDevice {
//     fn ifname(&self) -> &str {
//         self.dev.name()
//     }
// }

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let filter =
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info,urtcp=debug,tun_demo=debug".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .with_target(true)
        .with_level(true)
        .init();

    // You can tweak these without recompiling:
    let local_ip = [10, 10, 0, 1];
    let peer_ip = [10, 10, 0, 2];
    let mtu = 1500;
    let tun_name_hint = "utun"; // macOS ignores explicit number; "utun" = next free

    info!("starting tun demo");

    // Create TUN/utun
    let tun = TunDevice::new(tun_name_hint, mtu)?;
    let ifname = tun.ifname().to_string();
    info!(ifname, mtu, "created TUN interface");

    // Configure OS-side interface
    #[cfg(target_os = "macos")]
    {
        let local = format!(
            "{}.{}.{}.{}",
            local_ip[0], local_ip[1], local_ip[2], local_ip[3]
        );
        let peer = format!(
            "{}.{}.{}.{}",
            peer_ip[0], peer_ip[1], peer_ip[2], peer_ip[3]
        );
        info!(ifname, %local, %peer, "configuring utun with ifconfig");

        // ifconfig utunN <local> <peer> up
        let status = Command::new("/sbin/ifconfig")
            .arg(ifname)
            .arg(&local)
            .arg(&peer)
            .arg("up")
            .status();

        match status {
            Ok(s) if s.success() => info!("ifconfig succeeded"),
            Ok(s) => warn!(code=?s.code(), "ifconfig exited with non-zero status"),
            Err(e) => warn!(%e, "failed to run ifconfig (are you root?)"),
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        // On Linux or others, do it manually (example shown for Linux):
        let local = format!(
            "{}.{}.{}.{}",
            local_ip[0], local_ip[1], local_ip[2], local_ip[3]
        );
        let peer = format!(
            "{}.{}.{}.{}",
            peer_ip[0], peer_ip[1], peer_ip[2], peer_ip[3]
        );
        info!("bring the interface up manually (Linux example):");
        println!("  sudo ip addr add {}/24 dev {}", local, ifname);
        println!("  sudo ip link set {} up", ifname);
        println!("Peer example address: {}", peer);
    }

    // Spin up the stack on this TUN
    let cfg = StackConfig {
        local_ip,
        ttl: 64,
        ident_seed: 1,
        mtu,
    };

    let stack = Stack::new(tun, cfg.clone());
    let ctrl = stack.control();
    tokio::spawn(async move {
        if let Err(e) = stack.run().await {
            error!(%e, "stack exited with error");
        }
    });

    // info!(
    //     &ifname,
    //     "TUN interface ready; attempting connect after a short delay…"
    // );
    info!("TUN interface ready; attempting connect after a short delay…");
    sleep(Duration::from_secs(2)).await;

    // Try a connect to the peer IP on port 80.
    // NOTE: This will only fully succeed once your TCP state machine is implemented.
    let local = TcpSocketAddr {
        ip: cfg.local_ip,
        port: 50_000,
    };
    let remote = TcpSocketAddr {
        ip: peer_ip,
        port: 80,
    };
    info!(?local, ?remote, "TcpStream::connect()");
    match TcpStream::connect(ctrl.clone(), local, remote).await {
        Ok(mut stream) => {
            info!("connect() returned Ok (handshake may be stubbed)");
            let payload = b"hello over TUN".to_vec();
            debug!(len = payload.len(), "sending payload");
            if let Err(e) = stream.write_all(payload).await {
                warn!(%e, "write_all returned error (expected until engine is done)");
            }
        }
        Err(e) => error!(%e, "connect() failed"),
    }

    info!("tun_demo done");
    Ok(())
}
