use anyhow::Result;

use crate::ipv4::handle_ipv4;
use crate::netinet::{ip, IPVERSION};
use crate::tun::TunContext;

pub fn get_recv_ip(tun: &TunContext) -> &mut ip {
    unsafe { (tun.recv_buf as *mut ip).as_mut().unwrap_unchecked() }
}

pub async fn handle_ip(tun: &TunContext) -> Result<()> {
    let ip = get_recv_ip(tun);
    match ip.ip_v() {
        IPVERSION => handle_ipv4(tun, ip).await?,
        6 => tun.log("Got IPv6 pkt".to_string()),
        _ => tun.log(format!("Received unknown IP version {}", ip.ip_v())),
    };
    Ok(())
}
