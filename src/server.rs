use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use anyhow::Context;
use bytes::{Bytes, BytesMut};
use log::{debug, info};
use tokio::net::UdpSocket;

use crate::config::ServerConfig;
use crate::handler::parse_and_handle_request;

pub struct DNSServer {
    config: ServerConfig,
}

impl DNSServer {
    pub async fn listen(self) -> anyhow::Result<()> {
        let local_addr = SocketAddr::new(IpAddr::from([0u8, 0u8, 0u8, 0u8]), self.config.port);
        debug!("Binding to UDP: {}", local_addr);
        let socket = UdpSocket::bind(&local_addr)
            .await
            .context("Failed to bind to port")?;
        let socket = Arc::new(socket);
        info!("Bound to UDP: {}", local_addr);

        loop {
            let mut read_buffer = BytesMut::new();
            let (len, addr) = socket
                .recv_buf_from(&mut read_buffer)
                .await
                .context("Failed to read data from socket")?;
            let read_buffer = read_buffer.freeze();
            debug!("Read {}b from {}", len, addr);
            let socket = socket.clone();
            tokio::spawn(async move { Self::handle_request(read_buffer, socket, addr).await });
        }
    }

    async fn handle_request(
        request_bytes: Bytes,
        socket: Arc<UdpSocket>,
        remote_addr: SocketAddr,
    ) -> anyhow::Result<()> {
        debug!("Handling request from {:?}", remote_addr);
        let response = parse_and_handle_request(request_bytes).await?;
        let response_bytes = response.to_bytes()?;
        socket.send_to(&response_bytes, remote_addr).await?;
        debug!("Done handling request from {:?}", remote_addr);
        Ok(())
    }

    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }
}
