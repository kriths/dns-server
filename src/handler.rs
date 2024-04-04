use bytes::Bytes;
use log::{debug, error, trace};

use crate::data::header::*;
use crate::data::request::DNSRequest;
use crate::data::response::DNSResponse;
use crate::resolver::resolve_upstream;

async fn handle_request(request: &DNSRequest) -> anyhow::Result<DNSResponse> {
    // todo: check overrides
    // todo: check cache
    let upstream_response = resolve_upstream(request).await?;
    trace!("Got upstream response {:?}", upstream_response);

    // todo: store in cache
    return Ok(upstream_response);
}

pub async fn parse_and_handle_request(request_bytes: Bytes) -> anyhow::Result<DNSResponse> {
    let request = DNSRequest::from_bytes(request_bytes)?;
    debug!(
        "Handling request {} with {} questions",
        request.header.identification, request.header.count_questions
    );
    trace!("Handling request {:?}", request);

    let response = handle_request(&request).await.unwrap_or_else(|err| {
        error!("Error while handling request: {:?}", err);
        DNSResponse::empty(DNSHeader {
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
        })
    });
    Ok(response)
}
