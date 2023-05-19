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

    #[derive(Debug, Deserialize)]
    pub struct BinanceResponse<D> {
        pub stream: WsStream,
        pub data: BinanceResponseData<D>,
    }

    #[derive(Debug, Deserialize)]
    pub struct BinanceResponseData<D> {
        #[serde(rename = "e")]
        pub event_type: String,

        #[serde(rename = "E")]
        pub event_time: u64,

        #[serde(rename = "s")]
        pub symbol: String,

        #[serde(flatten)]
        pub inner: D,
    }

    #[derive(Debug, Deserialize)]
    pub struct BinanceResponseDataKline {
        #[serde(rename = "o")]
        pub open: BigDecimal,

        #[serde(rename = "h")]
        pub high: BigDecimal,

        #[serde(rename = "l")]
        pub low: BigDecimal,

        #[serde(rename = "c")]
        pub close: BigDecimal,
    }

    pub type KlineResponse = BinanceResponse<BinanceResponseDataKline>;

    #[derive(Debug)]
    pub struct StreamRequestAST<'a> {
        pub request: ast::ParserAST<'a>,
        pub candle_interval: String,
    }
}
