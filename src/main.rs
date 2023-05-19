mod ast;
mod models;

#[macro_use]
extern crate log;

use futures_util::{SinkExt, StreamExt};
use models::ws;
use std::collections::HashMap;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use warp::{ws::Ws, Filter, Rejection, Reply};

use crate::models::ws::KlineResponse;

const BINANCE_WS_URL: &str = "wss://fstream.binance.com/stream";

async fn handle_ws(ws: Ws) -> Result<impl Reply, Rejection> {
    Ok(ws.on_upgrade(move |websocket| async move {
        // let mut rates = HashMap::<String, ws::KlineResponse>::new();
        // let (sink, mut stream) = websocket.split();
        // let (binance_ws_stream, _) = connect_async(BINANCE_WS_URL)
        //     .await
        //     .expect("binance connection failed");

        // let (mut binance_sink, mut binance_stream) = binance_ws_stream.split();

        // if let Ok(body) = stream.next().await.unwrap() {
        //     let ws_request =
        //         serde_json::from_str::<ws::RawRequest>(body.to_str().unwrap()).unwrap();
        //     let parsed_request = ast::parse_stream_request(&ws_request.stream).unwrap();
        //     let binance_request = serde_json::to_string(&ws::BinanceRequest {
        //         id: ws_request.id,
        //         method: ws::Method::Subscribe,
        //         params: ast::get_all_variables_from_ast(&parsed_request.request),
        //     })
        //     .unwrap();

        //     binance_sink
        //         .send(Message::text(binance_request))
        //         .await
        //         .unwrap();

        //     while let Some(Ok(binance_response)) = binance_stream.next().await {
        //         let raw_resp = binance_response.into_text().unwrap();
        //         println!("{raw_resp}\n********************************");
        //         let response = serde_json::from_str::<KlineResponse>(&raw_resp).unwrap();
        //         let pair = response.stream.split('@').next().unwrap().to_string();

        //         rates.insert(pair, response);
        //         ast::eval_ast_with(&parsed_request.request, &rates);
        //     }
        // }
        //stream.forward(sink).await.unwrap();
    }))
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    // ast::parse_stream_request("(btcusdt+ethusdt*ltcbtc)/bnbusdt@1m");
    // return Ok(());

    // pretty_env_logger::init();

    // let routes = warp::path("ws").and(warp::ws()).and_then(handle_ws);

    // warp::serve(routes.with(warp::log("ws")))
    //     .run(([0, 0, 0, 0], 8080))
    //     .await;

    // Ok(())
    use arithmetic_parser::{
        grammars::{F32Grammar, Parse, Untyped},
        Expr, FnDefinition, LocatedSpan, LvalueLen, NomResult, Statement,
    };

    let program = String::from(
        r#"
        // This is a comment.
        x = 1 + 2.5 * 3 + sin(a^3 / b^2 /* another comment */);
        // Function declarations have syntax similar to Rust closures.
        some_function = |a, b| (a + b, a - b);
        other_function = |x| {
            r = min(rand(), 0.5);
            r * x
        };
        // Tuples and blocks are supported and have a similar syntax to Rust.
        (y, z) = some_function({ x = x - 0.5; x }, x);
        other_function(y - z)
    "#,
    );

    fn f(s: &str) -> LocatedSpan<&str, Statement<'_, Untyped<F32Grammar>>> {
        Untyped::<F32Grammar>::parse_statements(s)
            .unwrap()
            .statements
            .pop()
            .unwrap()
    }
    f(&program);
    // First statement is an assignment.
    Ok(())
}
