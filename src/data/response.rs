use bytes::{Bytes, BytesMut};

use crate::data::header::{DNSHeader, REQUEST_HEADER_SIZE};

#[derive(Debug)]
pub struct DNSResponse {
    pub header: DNSHeader,
}

impl DNSResponse {
    pub fn to_bytes(self) -> anyhow::Result<Bytes> {
        let expected_size = REQUEST_HEADER_SIZE; // todo: body?
        let mut bytes = BytesMut::with_capacity(expected_size);
        self.header.write_as_bytes(&mut bytes);
        // todo: write body
        Ok(bytes.freeze())
    }
}
