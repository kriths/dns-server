use anyhow::bail;
use bytes::{BufMut, BytesMut};

use crate::data::sizes::REQUEST_HEADER_SIZE;

#[derive(Debug, PartialEq)]
pub enum HeaderFlagQR {
    Query,
    Reply,
}

impl HeaderFlagQR {
    fn from_flags(flags: u16) -> Self {
        if (flags >> 15) & 1 == 0 {
            Self::Query
        } else {
            Self::Reply
        }
    }

    fn to_mask(&self) -> u16 {
        match self {
            HeaderFlagQR::Query => 0b0000_0000_0000_0000,
            HeaderFlagQR::Reply => 0b1000_0000_0000_0000,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum HeaderFlagOpCode {
    Query,
    IQuery,
    Status,
}

impl HeaderFlagOpCode {
    fn from_flags(flags: u16) -> Self {
        let code_field = (flags >> 11) & 0b1111;
        match code_field {
            1 => Self::IQuery,
            2 => Self::Status,
            _ => Self::Query,
        }
    }

    fn to_mask(&self) -> u16 {
        let id: u16 = match self {
            HeaderFlagOpCode::Query => 0,
            HeaderFlagOpCode::IQuery => 1,
            HeaderFlagOpCode::Status => 2,
        };
        return id << 11;
    }
}

#[derive(Debug, PartialEq)]
pub enum ResponseCode {
    NoError,
    FormatError,
    ServerFail,
    NonExistentDomain,
    NotImplemented,
    Refused,
    Unknown,
}

impl ResponseCode {
    fn from_flags(flags: u16) -> Self {
        match (flags & 0b1111) as u8 {
            0 => Self::NoError,
            1 => Self::FormatError,
            2 => Self::ServerFail,
            3 => Self::NonExistentDomain,
            4 => Self::NotImplemented,
            5 => Self::Refused,
            6..=0b1111 => Self::Unknown,
            _ => unreachable!(),
        }
    }

    fn to_mask(&self) -> u16 {
        match self {
            ResponseCode::NoError => 0,
            ResponseCode::FormatError => 1,
            ResponseCode::ServerFail => 2,
            ResponseCode::NonExistentDomain => 3,
            ResponseCode::NotImplemented => 4,
            ResponseCode::Refused => 5,
            ResponseCode::Unknown => 15,
        }
    }
}

#[derive(Debug)]
pub struct DNSHeader {
    pub identification: u16,

    pub msg_type: HeaderFlagQR,
    pub opcode: HeaderFlagOpCode,
    pub authoritative: bool,
    pub truncation: bool,
    pub recursion_desired: bool,
    pub recursion_available: bool,
    pub response_code: ResponseCode,

    pub count_questions: u16,
    pub count_answers: u16,
    pub count_authorities: u16,
    pub count_additional: u16,
}

impl DNSHeader {
    pub(crate) fn write_as_bytes(&self, output: &mut BytesMut) {
        assert!(output.is_empty()); // Header must be the first thing to write
        let mut flags =
            self.msg_type.to_mask() | self.opcode.to_mask() | self.response_code.to_mask();
        if self.authoritative {
            flags |= 0b0000_0100_0000_0000;
        }
        if self.truncation {
            flags |= 0b0000_0010_0000_0000;
        }
        if self.recursion_desired {
            flags |= 0b0000_0001_0000_0000;
        }
        if self.recursion_available {
            flags |= 0b0000_0000_1000_0000;
        }

        output.put_u16(self.identification);
        output.put_u16(flags);
        output.put_u16(self.count_questions);
        output.put_u16(self.count_answers);
        output.put_u16(self.count_authorities);
        output.put_u16(self.count_additional);
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        if bytes.len() != REQUEST_HEADER_SIZE {
            bail!("Request header has invalid length");
        }

        let flags = u16::from_be_bytes([bytes[2], bytes[3]]);
        Ok(Self {
            identification: u16::from_be_bytes([bytes[0], bytes[1]]),
            msg_type: HeaderFlagQR::from_flags(flags),
            opcode: HeaderFlagOpCode::from_flags(flags),
            authoritative: (flags >> 10) & 1 == 1,
            truncation: (flags >> 9) & 1 == 1,
            recursion_desired: (flags >> 8) & 1 == 1,
            recursion_available: (flags >> 7) & 1 == 1,
            response_code: ResponseCode::from_flags(flags),
            count_questions: u16::from_be_bytes([bytes[4], bytes[5]]),
            count_answers: u16::from_be_bytes([bytes[6], bytes[7]]),
            count_authorities: u16::from_be_bytes([bytes[8], bytes[9]]),
            count_additional: u16::from_be_bytes([bytes[10], bytes[11]]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_from_bytes_reject_invalid_length() {
        let bytes = &[0u8];
        let res_header = DNSHeader::from_bytes(bytes);
        assert!(res_header.is_err());
    }

    #[test]
    fn header_from_bytes_parses_identification() {
        let bytes = &[
            0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let header = DNSHeader::from_bytes(bytes).unwrap();
        assert_eq!(0x1234, header.identification);
    }

    #[test]
    fn header_from_bytes_parses_default_request_flags() {
        let bytes = &[
            0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let header = DNSHeader::from_bytes(bytes).unwrap();
        assert_eq!(HeaderFlagQR::Query, header.msg_type);
        assert_eq!(HeaderFlagOpCode::Query, header.opcode);
        assert_eq!(false, header.authoritative);
        assert_eq!(false, header.truncation);
        assert_eq!(true, header.recursion_desired);
        assert_eq!(false, header.recursion_available);
        assert_eq!(ResponseCode::NoError, header.response_code);
    }

    #[test]
    fn header_from_bytes_parses_count_questions() {
        let bytes = &[
            0x12, 0x34, 0x01, 0x00, 0x11, 0x11, 0x22, 0x22, 0x33, 0x33, 0x44, 0x44,
        ];
        let header = DNSHeader::from_bytes(bytes).unwrap();
        assert_eq!(0x1111, header.count_questions);
    }

    #[test]
    fn header_from_bytes_parses_count_answers() {
        let bytes = &[
            0x12, 0x34, 0x01, 0x00, 0x11, 0x11, 0x22, 0x22, 0x33, 0x33, 0x44, 0x44,
        ];
        let header = DNSHeader::from_bytes(bytes).unwrap();
        assert_eq!(0x2222, header.count_answers);
    }

    #[test]
    fn header_from_bytes_parses_count_authorities() {
        let bytes = &[
            0x12, 0x34, 0x01, 0x00, 0x11, 0x11, 0x22, 0x22, 0x33, 0x33, 0x44, 0x44,
        ];
        let header = DNSHeader::from_bytes(bytes).unwrap();
        assert_eq!(0x3333, header.count_authorities);
    }

    #[test]
    fn header_from_bytes_parses_count_additional() {
        let bytes = &[
            0x12, 0x34, 0x01, 0x00, 0x11, 0x11, 0x22, 0x22, 0x33, 0x33, 0x44, 0x44,
        ];
        let header = DNSHeader::from_bytes(bytes).unwrap();
        assert_eq!(0x4444, header.count_additional);
    }
}
