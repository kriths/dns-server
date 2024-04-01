use anyhow::bail;
use bytes::Bytes;

use crate::data::domain_name::DomainName;
use crate::data::header::{DNSHeader, REQUEST_HEADER_SIZE};
use crate::data::record_type::RecordType;

/// DNSQuestion represents a question to the server requesting a record
/// of a specific type for a given domain name. It is encoded in the following
/// format:
/// - Domain name: variable size, see below
/// - Type of the requested record: 2 bytes, see [RecordType]
/// - Class of the record: We'll always assume "Internet"
#[derive(Debug)]
pub struct DNSQuestion {
    pub record_type: RecordType,
    pub domain_name: String,
}

impl DNSQuestion {
    fn parse(bytes: &[u8]) -> anyhow::Result<Vec<DNSQuestion>> {
        let mut questions = Vec::new();
        let mut i: usize = 0;
        while i < bytes.len() {
            let (bytes_read, domain_name) = DomainName::try_from(&bytes[i..])?;
            let domain_name = domain_name.to_string()?;
            i += bytes_read; // Increment pointer past domain name

            let record_type_id = u16::from_be_bytes([bytes[i], bytes[i + 1]]);
            let record_type = RecordType::try_from(record_type_id)?;
            questions.push(DNSQuestion {
                record_type,
                domain_name,
            });

            i += 2; // Increment pointer past record type
            i += 2; // Increment pointer past ignored record class
        }

        Ok(questions)
    }
}

/// A DNS request starts with a common DNS header (the same format is used
/// for requests and replies) with a fixed size of 12 bytes. See [DNSHeader].
///
/// It is then followed by a sequence of [DNSQuestion] to the server. The
/// questions are directly appended to each other without any separation.
/// I.e. the length of a single question segment can only be determined
/// by parsing it.
#[derive(Debug)]
pub struct DNSRequest {
    pub header: DNSHeader,
    pub questions: Vec<DNSQuestion>,
}

impl DNSRequest {
    pub(crate) fn from_bytes(request_bytes: &Bytes) -> anyhow::Result<Self> {
        if request_bytes.len() < REQUEST_HEADER_SIZE {
            bail!("Invalid request header size")
        }

        let (header_bytes, question_bytes) = request_bytes.split_at(REQUEST_HEADER_SIZE);
        Ok(Self {
            header: DNSHeader::from_bytes(header_bytes)?,
            questions: DNSQuestion::parse(question_bytes)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::data::header::HeaderFlagQR;

    use super::*;

    #[test]
    fn request_from_bytes_fails_when_header_is_too_short() {
        let bytes = Bytes::from(vec![0x00]);
        assert!(DNSRequest::from_bytes(&bytes).is_err());
    }

    #[test]
    fn request_from_bytes_should_work_without_questions() {
        let bytes = Bytes::from(vec![
            0x12, 0x34, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]);
        let request = DNSRequest::from_bytes(&bytes).unwrap();
        assert_eq!(HeaderFlagQR::Query, request.header.msg_type);
        assert!(request.questions.is_empty());
    }

    #[test]
    fn request_from_bytes_should_work_with_one_question() {
        let bytes = Bytes::from(vec![
            0x12, 0x34, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x7a,
            0x7a, 0x7a, 0x02, 0x41, 0x41, 0x00, 0x00, 0x01, 0x00, 0x01,
        ]);
        let request = DNSRequest::from_bytes(&bytes).unwrap();
        assert_eq!(1, request.questions.len());
    }
}
