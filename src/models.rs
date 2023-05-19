pub mod ws {
    use crate::ast;
    use bigdecimal::BigDecimal;
    use serde::{Deserialize, Serialize};

    pub type WsStream = String;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum Method {
        Subscribe,
    }

    #[derive(Debug, Deserialize)]
    pub struct RawRequest {
        pub id: u64,
        pub method: Method,
        pub stream: String,
    }

    #[derive(Debug, Serialize)]
    pub struct BinanceRequest {
        pub id: u64,
        pub method: Method,
        pub params: Vec<WsStream>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct BinanceResponse {
        pub stream: WsStream,
        pub data: BinanceResponseData,
    }

    #[derive(Clone, Debug, Serialize, Default)]
    pub struct ClientResponse {
        pub stream: WsStream,
        pub data: Rates,
    }

    impl From<BinanceResponse> for ClientResponse {
        fn from(resp: BinanceResponse) -> Self {
            Self {
                stream: resp.stream,
                data: resp.data.k,
            }
        }
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct BinanceResponseData {
        #[serde(rename = "e")]
        pub event_type: String,

        #[serde(rename = "E")]
        pub event_time: u64,

        #[serde(rename = "s")]
        pub symbol: String,

        pub k: Rates,
    }

    #[derive(Clone, Debug, Serialize, Deserialize, Default)]
    pub struct Rates {
        #[serde(rename = "t")]
        pub kline_start_time: u64,

        #[serde(rename = "o")]
        pub open: BigDecimal,

        #[serde(rename = "h")]
        pub high: BigDecimal,

        #[serde(rename = "l")]
        pub low: BigDecimal,

        #[serde(rename = "c")]
        pub close: BigDecimal,
    }

    #[derive(Debug)]
    pub struct StreamRequestAST<'a> {
        pub request: ast::ParserAST<'a>,
        pub candle_interval: String,
    }
}
