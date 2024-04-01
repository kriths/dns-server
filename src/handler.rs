use bytes::Bytes;
use log::trace;

use crate::data::header::*;
use crate::data::request::DNSRequest;
use crate::data::response::DNSResponse;

pub async fn parse_and_handle_request(request_bytes: Bytes) -> anyhow::Result<DNSResponse> {
    let request = DNSRequest::from_bytes(request_bytes)?;
    trace!("Handling request {:?}", request);

    Ok(DNSResponse {
        header: DNSHeader {
            identification: request.header.identification,
            msg_type: HeaderFlagQR::Reply,
            opcode: request.header.opcode,
            authoritative: request.header.authoritative,
            truncation: false,
            recursion_desired: false,
            recursion_available: false,
            response_code: ResponseCode::ServerFail,
            count_questions: 0,
            count_answers: 0,
            count_authorities: 0,
            count_additional: 0,
        },
    })
}
