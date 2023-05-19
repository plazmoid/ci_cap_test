mod ast;
mod error;
mod models;

#[macro_use]
extern crate log;

use error::Error;
use futures_util::{SinkExt, StreamExt};
use models::ws;
use std::collections::HashMap;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use warp::{
    ws::{Message as WarpMessage, Ws},
    Filter, Rejection, Reply,
};

const BINANCE_WS_URL: &str = "wss://fstream.binance.com/stream";

async fn handle_ws(ws: Ws) -> Result<impl Reply, Rejection> {
    Ok(ws.on_upgrade(move |websocket| async move {
        let mut rates = HashMap::<String, ws::ClientResponse>::new();
        let (mut sink, mut stream) = websocket.split();
        let (binance_ws_stream, _) = connect_async(BINANCE_WS_URL)
            .await
            .expect("binance connection failed");

        let (mut binance_sink, mut binance_stream) = binance_ws_stream.split();

        if let Ok(body) = stream.next().await.unwrap() {
            let ws_request =
                serde_json::from_str::<ws::RawRequest>(body.to_str().unwrap()).unwrap();
            let parsed_request = ast::parse_stream_request(&ws_request.stream).unwrap();
            let all_required_pairs = ast::get_all_variables_from_ast(&parsed_request.request);
            let binance_request = serde_json::to_string(&ws::BinanceRequest {
                id: ws_request.id,
                method: ws::Method::Subscribe,
                params: all_required_pairs
                    .iter()
                    .map(|pair| format!("{pair}@kline_{}", parsed_request.candle_interval))
                    .collect(),
            })
            .unwrap();

            binance_sink
                .send(Message::text(binance_request))
                .await
                .unwrap();

            while let Some(Ok(binance_response)) = binance_stream.next().await {
                let raw_resp = binance_response.into_text().unwrap();

                if raw_resp.contains(r#""result":null"#) {
                    continue;
                }

                let response = serde_json::from_str::<ws::BinanceResponse>(&raw_resp).unwrap();
                let pair = response.stream.split('@').next().unwrap().to_string();

                rates.insert(pair, ws::ClientResponse::from(response));

                if rates.len() == all_required_pairs.len() {
                    let mut result_rates =
                        ast::eval_ast_with(&parsed_request.request, &rates).unwrap();
                    result_rates.stream = ws_request.stream.clone();

                    sink.send(WarpMessage::text(
                        serde_json::to_string(&result_rates).unwrap(),
                    ))
                    .await
                    .unwrap();
                }
            }
        }
    }))
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    pretty_env_logger::init();

    let routes = warp::path("ws").and(warp::ws()).and_then(handle_ws);

    warp::serve(routes.with(warp::log("ws")))
        .run(([0, 0, 0, 0], 8080))
        .await;

    Ok(())
}
