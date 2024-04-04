use anyhow::bail;
use bytes::{Bytes, BytesMut};

use crate::data::header::DNSHeader;
use crate::data::sizes::REQUEST_HEADER_SIZE;

#[derive(Debug)]
pub struct DNSResponse {
    pub header: DNSHeader,
    raw_bytes: Option<Bytes>,
}

impl DNSResponse {
    fn serialize(self) -> anyhow::Result<Bytes> {
        let expected_size = REQUEST_HEADER_SIZE; // todo: body?
        let mut bytes = BytesMut::with_capacity(expected_size);
        self.header.write_as_bytes(&mut bytes);
        // todo: write body
        Ok(bytes.freeze())
    }

    pub fn to_bytes(self) -> anyhow::Result<Bytes> {
        match self.raw_bytes {
            Some(bytes) => Ok(bytes),
            None => self.serialize(),
        }
    }

    pub(crate) fn empty(header: DNSHeader) -> Self {
        DNSResponse {
            header,
            raw_bytes: None,
        }
    }

    pub(crate) fn from_bytes(response_bytes: Bytes) -> anyhow::Result<Self> {
        if response_bytes.len() < REQUEST_HEADER_SIZE {
            bail!("Invalid request header size")
        }

        let (header_bytes, _) = response_bytes.split_at(REQUEST_HEADER_SIZE);
        // todo parse body
        Ok(Self {
            header: DNSHeader::from_bytes(header_bytes)?,
            raw_bytes: Some(response_bytes),
        })
    }
}
