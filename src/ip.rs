use anyhow::Result;

use crate::icmp::handle_icmp;
use crate::netinet::{ip, IPPROTO_ICMP};
use crate::tun::TunContext;

//pub fn calc_inet_checksum(data: *const u8, size: usize) -> Result<u16> {}

pub fn get_recv_ip(tun: &TunContext) -> &ip {
    unsafe { (tun.recv_buf as *const ip).as_ref().unwrap_unchecked() }
}

pub async fn handle_ip(tun: &TunContext) -> Result<()> {
    let ip = get_recv_ip(tun);
    match ip.ip_v() {
        4 => handle_ipv4(tun, ip).await?,
        6 => tun.log("Got IPv6 pkt".to_string()),
        _ => tun.log(format!("Received unknown IP version {}", ip.ip_v())),
    };
    Ok(())
}

pub async fn handle_ipv4(tun: &TunContext, ip: &ip) -> Result<()> {
    match ip.ip_p as u32 {
        IPPROTO_ICMP => handle_icmp(tun, ip).await?,
        _ => (),
    };
    Ok(())
}
