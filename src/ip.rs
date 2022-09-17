#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use anyhow::Result;

include!(concat!(env!("OUT_DIR"), "/ip_bindings.rs"));

//pub fn calc_inet_checksum(data: *const u8, size: usize) -> Result<u16> {}
