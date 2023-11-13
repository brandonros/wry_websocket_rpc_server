#![allow(non_camel_case_types, non_snake_case)]

use std::error::Error;

use async_tungstenite::tungstenite::Message;
use futures::{channel::mpsc::UnboundedSender, SinkExt};

use crate::{requests::{SumRequest, Request}, responses::{SumResponse, Response}, messagepack_helpers};

pub async fn on_sum_request(
    parsed_request_body: SumRequest,
) -> Result<SumResponse, Box<dyn Error>> {
    // log request body
    log::info!("{parsed_request_body:?}");
    // calculate
    let response_body = SumResponse {
        sum: parsed_request_body.operands.iter().sum()
    };
    // log response
    log::info!("response_body = {response_body:?}");
    // return response
    Ok(response_body)
}

pub async fn on_client_message(mut tx: UnboundedSender<Message>, incoming_msg: Vec<u8>) {
    log::info!("on_client_message: incoming_msg = {incoming_msg:02x?}");
    match messagepack_helpers::deserialize::<Request>(&incoming_msg) {
        Ok(Request::Sum {
            request_id,
            body: parsed_request_body,
        }) => {
            let response = Response::Sum {
                request_id,
                body: on_sum_request(parsed_request_body)
                    .await
                    .unwrap(),
            };
            let serialized_response_body = messagepack_helpers::serialize(&response).unwrap();
            let message = Message::Binary(serialized_response_body);
            tx.send(message).await.unwrap();
        }
        Err(_e) => todo!(),
    }
}
