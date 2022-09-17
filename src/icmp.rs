use std::{convert::TryInto, mem::size_of, net::Ipv4Addr, ptr::write_bytes};

use anyhow::Result;

use crate::{
    ipv4::{calc_inet_checksum, make_ipv4_reply, send_ipv4},
    netinet::{
        icmphdr, ip, ICMP_DEST_UNREACH, ICMP_ECHO, ICMP_EXC_TTL, ICMP_HOST_UNREACH,
        ICMP_PORT_UNREACH, ICMP_TIME_EXCEEDED, IPPROTO_ICMP,
    },
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
    let src_addr = Ipv4Addr::from(u32::from_be(ip.ip_src.s_addr));
    let mut dst_addr = Ipv4Addr::from(u32::from_be(ip.ip_dst.s_addr));
    let ttl = ip.ip_ttl;
    tun.log(format!(
        "Got ICMP echo message to {} with TTL {} from {}",
        dst_addr, ttl, src_addr
    ));
    if ttl == 100 {
        reply_icmp_error(tun, ip, dst_addr, src_addr, IcmpErrorMode::PortUnreachable).await?;
    } else {
        let mut dst_oct = dst_addr.octets();
        dst_oct[3] -= ttl;
        dst_addr = Ipv4Addr::from(dst_oct);
        reply_icmp_error(tun, ip, dst_addr, src_addr, IcmpErrorMode::TtlExceeded).await?;
    }
    Ok(())
}

pub enum IcmpErrorMode {
    HostUnreachable,
    PortUnreachable,
    TtlExceeded,
}

pub async fn reply_icmp_error(
    tun: &TunContext,
    ip: &ip,
    src_addr: Ipv4Addr,
    dst_addr: Ipv4Addr,
    mode: IcmpErrorMode,
) -> Result<()> {
    let (reply_ip, reply_icmp) = prepare_icmp_err_reply(tun);
    let icmp_payload_size = (ip.ip_hl() * 4) as usize + 8;
    let payload_size = size_of::<icmphdr>() + icmp_payload_size;
    let packet_size = size_of::<ip>() + payload_size;
    make_ipv4_reply(
        reply_ip,
        IPPROTO_ICMP as u8,
        ip.ip_id,
        payload_size,
        src_addr,
        dst_addr,
    )?;
    let type_ = match mode {
        IcmpErrorMode::HostUnreachable => ICMP_DEST_UNREACH as u8,
        IcmpErrorMode::PortUnreachable => ICMP_DEST_UNREACH as u8,
        IcmpErrorMode::TtlExceeded => ICMP_TIME_EXCEEDED as u8,
    };
    let code = match mode {
        IcmpErrorMode::HostUnreachable => ICMP_HOST_UNREACH as u8,
        IcmpErrorMode::PortUnreachable => ICMP_PORT_UNREACH as u8,
        IcmpErrorMode::TtlExceeded => ICMP_EXC_TTL as u8,
    };
    make_icmp_err_reply(reply_icmp, type_, code, icmp_payload_size)?;
    send_ipv4(tun, reply_ip, packet_size).await?;
    Ok(())
}

pub fn prepare_icmp_err_reply(tun: &TunContext) -> (&mut ip, &mut icmphdr) {
    unsafe {
        let reply_base = tun.recv_buf.offset(-ICMP_REPLY_FULL_SIZE);
        write_bytes(reply_base as *mut u8, 0, ICMP_REPLY_FULL_SIZE as usize);
        let reply_ip = (reply_base as *mut ip).as_mut().unwrap();
        let reply_icmp = (reply_base.offset(size_of::<ip>() as isize) as *mut icmphdr)
            .as_mut()
            .unwrap();
        (reply_ip, reply_icmp)
    }
}

pub fn make_icmp_err_reply(
    icmp: &mut icmphdr,
    type_: u8,
    code: u8,
    payload_size: usize,
) -> Result<()> {
    icmp.type_ = type_;
    icmp.code = code;
    icmp.checksum = unsafe {
        calc_inet_checksum(
            icmp as *const icmphdr as *const u8,
            size_of::<icmphdr>() + payload_size,
        )
    };
    Ok(())
}
