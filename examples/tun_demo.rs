use urtcp::device::tun_backend::TunDevice;
use urtcp::tcp::socket::{TcpSocketAddr, TcpStream};
use urtcp::{Stack, StackConfig};

use tokio::time::{Duration, sleep};
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    // initialise logging
    let filter =
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info,urtcp=debug,tun_demo=debug".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .with_target(true)
        .with_level(true)
        .init();

    info!("starting tun demo");

    // create/configure tun device (IPv4, /24 netmask)
    let mtu = 1500;
    let tun_name = "urtcp0";
    let tun_a = TunDevice::new(tun_name, mtu)?;
    debug!(tun_name, mtu, "created TUN device");

    // stack config — this IP is the urTCP side's "real" IPv4 on the TUN link
    let cfg = StackConfig {
        local_ip: [10, 10, 0, 1],
        ttl: 64,
        ident_seed: 1,
        mtu,
    };

    let stack = Stack::new(tun_a, cfg.clone());
    let ctrl = stack.control();
    tokio::spawn(async move {
        if let Err(e) = stack.run().await {
            error!(%e, "stack exited with error");
        }
    });

    info!(
        "TUN interface {} up, IP {}.{}.{}.{}",
        tun_name, cfg.local_ip[0], cfg.local_ip[1], cfg.local_ip[2], cfg.local_ip[3]
    );
    info!("configure your host to assign an IP to {}, e.g.:", tun_name);
    println!("  sudo ip addr add 10.10.0.2/24 dev {}", tun_name);
    println!("  sudo ip link set {} up", tun_name);

    // wait a bit for manual config before attempting connect
    sleep(Duration::from_secs(5)).await;

    let local = TcpSocketAddr {
        ip: cfg.local_ip,
        port: 50000,
    };
    let remote = TcpSocketAddr {
        ip: [10, 10, 0, 2],
        port: 80,
    };
    info!(?local, ?remote, "attempting TcpStream::connect");
    match TcpStream::connect(ctrl.clone(), local, remote).await {
        Ok(stream) => {
            info!("connect() returned Ok (handshake still stubbed)");
            let payload = b"hello over TUN".to_vec();
            debug!(len = payload.len(), "sending payload");
            if let Err(e) = stream.write_all(payload).await {
                warn!(%e, "write_all returned error");
            }
        }
        Err(e) => error!(%e, "connect() failed"),
    }

    Ok(())
}
