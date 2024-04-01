use anyhow::bail;

#[derive(Debug, PartialEq)]
pub(crate) enum RecordType {
    A,
    AAAA,
    CNAME,
    MX,
    NS,
    SOA,
    SRV,
    TXT,
}

impl TryFrom<u16> for RecordType {
    type Error = anyhow::Error;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::A),
            28 => Ok(Self::AAAA),
            5 => Ok(Self::CNAME),
            15 => Ok(Self::MX),
            2 => Ok(Self::NS),
            6 => Ok(Self::SOA),
            33 => Ok(Self::SRV),
            16 => Ok(Self::TXT),
            _ => bail!("Unknown record type id {}", value),
        }
    }
}

impl Into<u16> for RecordType {
    fn into(self) -> u16 {
        match self {
            RecordType::A => 1,
            RecordType::AAAA => 28,
            RecordType::CNAME => 5,
            RecordType::MX => 15,
            RecordType::NS => 2,
            RecordType::SOA => 6,
            RecordType::SRV => 33,
            RecordType::TXT => 16,
        }
    }
}
