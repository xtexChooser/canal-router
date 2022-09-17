use std::{convert::TryInto, mem::size_of, net::Ipv4Addr, ptr::write_bytes};

use anyhow::Result;

use crate::{
    netinet::{icmphdr, ip, ICMP_ECHO},
    tun::TunContext,
};

pub const ICMP_REPLY_FULL_SIZE: isize = (size_of::<ip>() + size_of::<icmphdr>()) as isize;

pub fn get_recv_icmp<'a>(tun: &'a TunContext, ip: &ip) -> &'a icmphdr {
    unsafe {
        (tun.recv_buf.offset((ip.ip_hl() * 4).try_into().unwrap()) as *const icmphdr)
            .as_ref()
            .unwrap_unchecked()
    }
}

pub async fn handle_icmp(tun: &TunContext, ip: &ip) -> Result<()> {
    let icmp = get_recv_icmp(tun, ip);
    match icmp.type_ as u32 {
        ICMP_ECHO => handle_icmp_echo_message(tun, ip, icmp).await?,
        _ => (),
    }
    Ok(())
}

pub async fn handle_icmp_echo_message(tun: &TunContext, ip: &ip, icmp: &icmphdr) -> Result<()> {
    let dest_addr = Ipv4Addr::from(u32::from_be(ip.ip_dst.s_addr));
    let ttl = ip.ip_ttl;
    tun.log(format!(
        "Got ICMP echo message to {} with TTL {}",
        dest_addr, ttl
    ));
    reply_icmp_unreachable(tun).await?;
    Ok(())
}

pub async fn reply_icmp_unreachable(tun: &TunContext) -> Result<()> {
    let (reply_ip, reply_icmp) = prepare_icmp_reply(tun);
    Ok(())
}

pub fn prepare_icmp_reply(tun: &TunContext) -> (&mut ip, &mut icmphdr) {
    unsafe {
        let reply_base = tun.recv_buf.offset(-ICMP_REPLY_FULL_SIZE);
        let reply_ip = (reply_base as *mut ip).as_mut().unwrap();
        let reply_icmp = (reply_base.offset(size_of::<icmphdr>() as isize) as *mut icmphdr)
            .as_mut()
            .unwrap();

        write_bytes(reply_base as *mut u8, 0, ICMP_REPLY_FULL_SIZE as usize);
        (reply_ip, reply_icmp)
    }
}
