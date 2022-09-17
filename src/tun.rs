use std::mem::size_of;

use anyhow::Result;
use tokio_tun::Tun;

use crate::ip;

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
            Box::try_new_uninit_slice(
                <i32 as TryInto<usize>>::try_into(tun.mtu().unwrap())? + REVERSE_BUFFER_HEADER,
            )?
            .assume_init()
        };
        let recv_buf = unsafe { buffer.as_ptr().offset(REVERSE_BUFFER_HEADER.try_into()?) };
        Ok(Self {
            tun,
            queue,
            buffer,
            recv_buf,
        })
    }

    fn log(&self, text: String) {
        println!("[{}] {}", self.queue, text)
    }

    pub async fn handle(mut self) -> Result<()> {
        self.log(format!("Poll"));
        loop {
            let size = self
                .tun
                .recv(&mut self.buffer[REVERSE_BUFFER_HEADER..])
                .await?;
            if size <= size_of::<ip::ip>() {
                self.log(format!("Received packet with too small size {}", size));
            } else {
                self.log(format!("Received packet {}", size));
                //self.handle_ip().await?;
            }
        }
    }
}
