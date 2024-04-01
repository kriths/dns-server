use anyhow::bail;
use bytes::Bytes;

use crate::data::header::{DNSHeader, REQUEST_HEADER_SIZE};

#[derive(Debug)]
pub struct DNSRequest {
    pub header: DNSHeader,
}

impl DNSRequest {
    pub(crate) fn from_bytes(mut request_bytes: Bytes) -> anyhow::Result<Self> {
        if request_bytes.len() < REQUEST_HEADER_SIZE {
            bail!("Invalid request header size")
        }

        let header_bytes = request_bytes.split_to(REQUEST_HEADER_SIZE);
        let header = DNSHeader::from_bytes(header_bytes);
        // todo: parse request body
        Ok(Self { header })
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;

    #[test]
    fn request_from_bytes_fails_when_header_is_too_short() {
        let bytes: Bytes = Bytes::from(vec![0x00]);
        assert!(DNSRequest::from_bytes(bytes).is_err());
    }
}
