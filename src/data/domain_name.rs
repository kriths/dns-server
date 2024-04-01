use anyhow::Context;
use bytes::{BufMut, BytesMut};

pub(crate) struct DomainName<'a> {
    parts: Vec<&'a [u8]>,
}

impl<'a> DomainName<'a> {
    pub(crate) fn to_string(&self) -> anyhow::Result<String> {
        // Start at length 32 to avoid the first couple re-allocations
        let mut bytes = BytesMut::with_capacity(32);
        for part in &self.parts[..self.parts.len() - 1] {
            bytes.put_slice(part);
            bytes.put_u8(b'.');
        }
        bytes.put_slice(self.parts[self.parts.len() - 1]);

        let as_str = String::from_utf8(bytes.to_vec())
            .context("Domain name contained broken utf-8 sequences")?;
        return Ok(as_str.to_lowercase());
    }

    pub(crate) fn try_from(bytes: &'a [u8]) -> anyhow::Result<(usize, DomainName<'a>)> {
        let mut parts = Vec::new();
        let mut binary_size: usize = 0;
        loop {
            let part_length = bytes[binary_size] as usize;
            binary_size += 1;
            if part_length == 0 {
                break;
            }

            parts.push(&bytes[binary_size..binary_size + part_length]);
            binary_size += part_length;
        }

        Ok((binary_size, Self { parts }))
    }
}

#[cfg(test)]
mod tests {
    use crate::data::domain_name::DomainName;

    #[test]
    fn parses_regular_domain_name() {
        let bytes = [0x03, 0x41, 0x41, 0x41, 0x02, 0x42, 0x42, 0x00];
        let (length, domain_name) = DomainName::try_from(&bytes).unwrap();
        assert_eq!(8, length);
        assert_eq!(2, domain_name.parts.len());
        assert_eq!(&[0x41, 0x41, 0x41], domain_name.parts[0]);
        assert_eq!(&[0x42, 0x42], domain_name.parts[1]);
    }

    #[test]
    fn parses_regular_domain_name_with_trailing_data() {
        let bytes = [
            0x03, 0x41, 0x41, 0x41, 0x02, 0x42, 0x42, 0x00, 0x01, 0x02, 0x03, 0x04,
        ];
        let (length, domain_name) = DomainName::try_from(&bytes).unwrap();
        assert_eq!(8, length);
        assert_eq!(2, domain_name.parts.len());
        assert_eq!(&[0x41, 0x41, 0x41], domain_name.parts[0]);
        assert_eq!(&[0x42, 0x42], domain_name.parts[1]);
    }

    #[test]
    fn rejects_invalid_domain_name() {
        let bytes = [0x03, 0x41, 0x80, 0x41, 0x00];
        let (_, domain_name) = DomainName::try_from(&bytes).unwrap();
        let as_str = domain_name.to_string();
        assert!(as_str.is_err());
    }
}
