use tokio::time::{Duration, sleep};
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;
use urtcp::device::LoopDevice;
use urtcp::tcp::socket::{TcpSocketAddr, TcpStream};
use urtcp::{Stack, StackConfig};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("starting loopback demo");

    let mtu = 1500;
    let (dev_a, dev_b) = LoopDevice::pair(1500);
    let cfg = StackConfig {
        local_ip: [10, 0, 0, 1],
        ttl: 64,
        ident_seed: 1,
        mtu: 1500,
    };
    let cfg_a = StackConfig {
        local_ip: [10, 0, 0, 1],
        ttl: 64,
        ident_seed: 1,
        mtu,
    };
    let stack_a = Stack::new(dev_a, cfg.clone());
    let ctrl_a = stack_a.control();
    info!(?cfg_a, "spawning stack A");
    tokio::spawn(async move {
        let _ = stack_a.run().await;
    });

    let stack_b = Stack::new(
        dev_b,
        StackConfig {
            local_ip: [10, 0, 0, 2],
            ..cfg
        },
    );

    let cfg_b = StackConfig {
        local_ip: [10, 0, 0, 2],
        ..cfg_a
    };
    let _ctrl_b = stack_b.control();
    info!(?cfg_b, "spawning stack B");
    tokio::spawn(async move {
        let _ = stack_b.run().await;
    });
    // attempt a connect from A -> B
    let local = TcpSocketAddr {
        ip: [10, 0, 0, 1],
        port: 50_000,
    };
    let remote = TcpSocketAddr {
        ip: [10, 0, 0, 2],
        port: 80,
    };
    info!(?local, ?remote, "attempting TcpStream::connect");
    // This will currently send a SYN and then stall (no TCP logic yet).
    // let _stream = TcpStream::connect(
    //     ctrl_a,
    //     TcpSocketAddr {
    //         ip: [10, 0, 0, 1],
    //         port: 50000,
    //     },
    //     TcpSocketAddr {
    //         ip: [10, 0, 0, 2],
    //         port: 80,
    //     },
    // )
    // .await?;
    match TcpStream::connect(ctrl_a.clone(), local, remote).await {
        Ok(stream) => {
            info!("connect() returned Ok (handshake logic may still be stubbed)");
            // optional: try a write to see the path; this will likely be queued or no-op in the skeleton
            let payload = b"hello from A".to_vec();
            debug!(len = payload.len(), "sending payload");
            if let Err(e) = stream.write_all(payload).await {
                warn!(%e, "write_all returned error (expected until TCP engine is implemented)");
            }
        }
        Err(e) => {
            error!(%e, "connect() failed");
        }
    }
    info!("sleeping a bit so background tasks can log…");
    sleep(Duration::from_secs(2)).await;

    info!("loopback demo done");

    Ok(())
}
