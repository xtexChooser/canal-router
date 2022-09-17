use std::{mem::size_of, os::unix::io::AsRawFd};

use anyhow::Result;
use tokio_tun::Tun;

use crate::{ip::handle_ip, netinet};

pub const REVERSE_BUFFER_HEADER: usize = 64;

pub struct TunContext {
    pub tun: Tun,
    pub queue: usize,
    pub buffer: Box<[u8]>,
    pub recv_buf: *const u8,
}

unsafe impl Send for TunContext {}
unsafe impl Sync for TunContext {}

impl TunContext {
    pub fn new(tun: Tun, queue: usize) -> Result<Self> {
        let buffer: Box<[u8]> = unsafe {
            Box::new_zeroed_slice(tun.mtu().unwrap() as usize + REVERSE_BUFFER_HEADER).assume_init()
        };
        let recv_buf = unsafe { buffer.as_ptr().offset(REVERSE_BUFFER_HEADER as isize) };
        Ok(Self {
            tun,
            queue,
            buffer,
            recv_buf,
        })
    }

    pub fn log(&self, text: String) {
        println!("[{}] {}", self.queue, text)
    }

    pub async fn handle(mut self) -> Result<()> {
        self.log(format!("Polling with fd {}", self.tun.as_raw_fd()));
        loop {
            let size = self
                .tun
                .recv(&mut self.buffer[REVERSE_BUFFER_HEADER..])
                .await?;
            if size <= size_of::<netinet::ip>() {
                self.log(format!("Received packet with too small size {}", size));
            } else {
                handle_ip(&self).await?;
            }
        }
    }
}
