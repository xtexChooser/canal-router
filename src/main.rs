use std::os::unix::prelude::AsRawFd;

use anyhow::Result;
use canal_router::{config::CONFIG, tun::TunContext};
use tokio::task::JoinSet;
use tokio_tun::TunBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let mut tuns = TunBuilder::new()
        .name(CONFIG.dev_name.as_str())
        .tap(false)
        .packet_info(false)
        .up()
        .try_build_mq(CONFIG.queues)
        .unwrap();

    println!("TUN device created with {} queues:", tuns.len());
    tuns.iter().for_each(|tun| {
        println!("- {}: FD: {}", tun.name(), tun.as_raw_fd());
    });

    let mut tasks = JoinSet::new();
    for i in 0..CONFIG.queues {
        let tun = tuns.swap_remove(0);
        tasks.spawn(TunContext::new(tun, i)?.handle());
    }

    while let Some(result) = tasks.join_next().await {
        result??;
    }

    Ok(())
}
