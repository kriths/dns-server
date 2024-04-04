use bytes::BytesMut;
use log::info;
use tokio::net::UdpSocket;
use tokio::time::Instant;

use crate::data::request::DNSRequest;
use crate::data::response::DNSResponse;
use crate::data::sizes::MAX_DNS_PACKET_SIZE;

const UPSTREAM: &'static str = "1.1.1.1:53"; // todo get from config

pub(crate) async fn resolve_upstream(request: &DNSRequest) -> anyhow::Result<DNSResponse> {
    let request_bytes = request.to_bytes()?;

    let start_time = Instant::now();
    let sock = UdpSocket::bind("0.0.0.0:0").await?;
    sock.send_to(request_bytes, UPSTREAM).await?;

    let mut response_buffer = BytesMut::with_capacity(MAX_DNS_PACKET_SIZE);
    let (len, _) = sock.recv_buf_from(&mut response_buffer).await?;
    let request_duration = start_time.elapsed();
    info!(
        "Received upstream {}b response in {}ms",
        len,
        request_duration.as_millis()
    );

    let response_buffer = response_buffer.freeze();
    DNSResponse::from_bytes(response_buffer)
}
