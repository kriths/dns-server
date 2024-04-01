use bytes::Bytes;
use log::{debug, trace};

use crate::data::header::*;
use crate::data::request::DNSRequest;
use crate::data::response::DNSResponse;

pub async fn parse_and_handle_request(request_bytes: Bytes) -> anyhow::Result<DNSResponse> {
    let request = DNSRequest::from_bytes(&request_bytes)?;
    debug!(
        "Handling request {} with {} questions",
        request.header.identification, request.header.count_questions
    );
    trace!("Handling request {:?}", request);
    // todo: check overrides
    // todo: check cache
    // todo: forward to upstream
    // todo: store in cache
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
