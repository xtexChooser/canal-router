use std::{mem::size_of, net::Ipv4Addr, path::PathBuf, slice};

use anyhow::Result;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{
    icmp::handle_icmp,
    netinet::{ip, IPPROTO_ICMP},
    tun::TunContext,
};

pub async fn handle_ipv4(tun: &TunContext, ip: &ip) -> Result<()> {
    match ip.ip_p as u32 {
        IPPROTO_ICMP => handle_icmp(tun, ip).await?,
        _ => (),
    };
    Ok(())
}

pub fn make_ipv4_reply(
    ip: &mut ip,
    protocol: u8,
    id: u16,
    payload_size: usize,
    src_addr: Ipv4Addr,
    dst_addr: Ipv4Addr,
) -> Result<()> {
    ip.set_ip_v(4);
    ip.set_ip_hl(size_of::<ip>() as u32 / 4);
    ip.ip_len = ((size_of::<ip>() + payload_size) as u16).to_be();
    ip.ip_ttl = 255;
    ip.ip_p = protocol;
    ip.ip_id = id;
    ip.ip_src.s_addr = u32::from(src_addr).to_be();
    ip.ip_dst.s_addr = u32::from(dst_addr).to_be();
    ip.ip_sum = unsafe { calc_inet_checksum(ip as *const ip as *const u8, size_of::<ip>()) };
    Ok(())
}

pub unsafe fn calc_inet_checksum(data: *const u8, size: usize) -> u16 {
    let mut checksum: u32 = 0;
    let data_u16 = data as *const u16;

    for i in 0..(size / 2) {
        checksum += *data_u16.offset(i as isize) as u32;
    }
    if size % 1 == 1 {
        checksum += *data.offset(size as isize) as u32;
    }
    reduce_inet_checksum(checksum)
}

fn reduce_inet_checksum(checksum: u32) -> u16 {
    let mut checksum = checksum;
    while (checksum >> 16) != 0 {
        checksum = (checksum & 0xffff) + (checksum >> 16);
    }
    !(checksum as u16)
}

pub async fn send_ipv4(tun: &TunContext, data: &mut ip, size: usize) -> Result<()> {
    let data = (|| unsafe { slice::from_raw_parts(data as *const ip as *const u8, size) })();
    File::create(PathBuf::from("a.bin"))
        .await?
        .write(data)
        .await?;
    tun.tun.send(data).await?;
    Ok(())
}
