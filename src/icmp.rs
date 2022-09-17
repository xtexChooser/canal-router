use std::{mem::size_of, net::Ipv4Addr};

use anyhow::{Context, Result};

use crate::{ip, tun::TunContext};

pub fn get_recv_icmp(tun: &TunContext) -> Result<&ip::icmphdr> {
    let ip = tun.get_recv_ip()?;
    Ok(unsafe {
        (tun.recv_buf.offset((ip.ip_hl() * 4).try_into()?) as *const ip::icmphdr)
            .as_ref()
            .context("")?
    })
}

pub async fn handle_icmp(tun: &TunContext) -> Result<()> {
    let ip = tun.get_recv_ip()?;
    let dst_addr = Ipv4Addr::from(ip.ip_dst.s_addr);
    let ttl = ip.ip_ttl;

    let icmp = get_recv_icmp(tun)?;
    match icmp.type_ {
        8 => {
            handle_icmp_echo_message(tun, dst_addr, ttl).await?;
        }
        _ => (),
    }
    Ok(())
}

pub async fn handle_icmp_echo_message(tun: &TunContext, addr: Ipv4Addr, ttl: u8) -> Result<()> {
    reply_icmp_unreachable(tun).await?;
    Ok(())
}

pub fn prepare_icmp_reply(tun: &TunContext) -> Result<(&mut ip::ip, &mut ip::icmphdr)> {
    let header_size = size_of::<ip::ip>() + size_of::<ip::icmphdr>();
    let reply_base = unsafe { tun.recv_buf.offset(-header_size.try_into()?) };
    let reply_ip = unsafe { (reply_base as *mut ip::ip).as_mut().context("")? };
    let reply_icmp = unsafe {
        (reply_base.offset(size_of::<ip::icmphdr>().try_into()?) as *mut ip::icmphdr)
            .as_mut()
            .context("")?
    };

    unsafe {
        reply_base.write_bytes(0, header_size);
    }
    Ok((reply_ip, reply_icmp))
}

pub async fn reply_icmp_unreachable(tun: &TunContext) -> Result<()> {
    let (reply_ip, reply_icmp) = prepare_icmp_reply(tun)?;
    Ok(())
}