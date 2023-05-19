use std::collections::HashMap;

use crate::{ws, Error};
use arithmetic_parser::{
    grammars::{F32Grammar, Parse, Untyped},
    BinaryOp, Expr, ExprType, LocatedSpan,
};

pub type ParserAST<'a> = LocatedSpan<&'a str, Expr<'a, Untyped<F32Grammar>>>;

pub fn parse_stream_request(s: &str) -> Result<ws::StreamRequestAST<'_>, Error> {
    let full_req = s.split('@').collect::<Vec<_>>();
    let req = full_req[0];
    let interval = full_req
        .get(1)
        .ok_or_else(|| Error::ParseError("no @interval found".to_string()))?;
    let block = Untyped::<F32Grammar>::parse_statements(req)
        .map_err(|e| Error::ParseError(e.to_string()))?;
    let expression = block
        .return_value
        .ok_or_else(|| Error::ParseError("no expressions found".to_string()))?;

    Ok(ws::StreamRequestAST {
        request: *expression,
        candle_interval: interval.to_string(),
    })
}

pub fn get_all_variables_from_ast(node: &ParserAST<'_>) -> Vec<String> {
    let mut variables = vec![];

    let mut process_node = |node: &ParserAST<'_>| match node.extra.ty() {
        ExprType::Binary => variables.extend(get_all_variables_from_ast(&*node)),
        ExprType::Variable => variables.push(node.fragment().to_string()),
        _ => (),
    };

    match &node.extra {
        Expr::Binary { lhs, rhs, .. } => {
            process_node(lhs);
            process_node(rhs)
        }
        _ => (),
    }

    variables
}

pub fn eval_ast_with(
    node: &ParserAST<'_>,
    vals: &HashMap<String, ws::KlineResponse>,
) -> Option<ws::KlineResponse> {
    let mut result: Option<ws::KlineResponse> = None;

    match &node.extra {
        Expr::Variable => {
            let pair = node.fragment().to_string();
            result = vals.get(&pair).cloned();
        }
        Expr::Binary { lhs, op, rhs } => {
            let lhs = eval_ast_with(&*lhs, vals)?;
            let rhs = eval_ast_with(&*rhs, vals)?;

            let mut kline_result: ws::KlineResponse = ws::KlineResponse::default();
            kline_result.data.event_time = u64::max(lhs.data.event_time, rhs.data.event_time);
            kline_result.data.event_type = lhs.data.event_type;
            kline_result.data.symbol = lhs.data.symbol;
            match op.extra {
                BinaryOp::Add => {
                    kline_result.data.k.open = lhs.data.k.open + rhs.data.k.open;
                    kline_result.data.k.high = lhs.data.k.high + rhs.data.k.high;
                    kline_result.data.k.low = lhs.data.k.low + rhs.data.k.low;
                    kline_result.data.k.close = lhs.data.k.close + rhs.data.k.close;
                }
                BinaryOp::Sub => {
                    kline_result.data.k.open = lhs.data.k.open - rhs.data.k.open;
                    kline_result.data.k.high = lhs.data.k.high - rhs.data.k.high;
                    kline_result.data.k.low = lhs.data.k.low - rhs.data.k.low;
                    kline_result.data.k.close = lhs.data.k.close - rhs.data.k.close;
                }
                BinaryOp::Mul => {
                    kline_result.data.k.open = lhs.data.k.open * rhs.data.k.open;
                    kline_result.data.k.high = lhs.data.k.high * rhs.data.k.high;
                    kline_result.data.k.low = lhs.data.k.low * rhs.data.k.low;
                    kline_result.data.k.close = lhs.data.k.close * rhs.data.k.close;
                }
                BinaryOp::Div => {
                    // todo /0
                    kline_result.data.k.open = lhs.data.k.open / rhs.data.k.open;
                    kline_result.data.k.high = lhs.data.k.high / rhs.data.k.high;
                    kline_result.data.k.low = lhs.data.k.low / rhs.data.k.low;
                    kline_result.data.k.close = lhs.data.k.close / rhs.data.k.close;
                }
                _ => unimplemented!(),
            }
            result = Some(kline_result);
        }
        _ => (),
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ws::KlineResponse;
    use bigdecimal::BigDecimal;

    #[test]
    fn get_vars() {
        let req = parse_stream_request("(btcusdt+ethusdt*ltcbtc)/bnbusdt@1m").unwrap();
        let result = get_all_variables_from_ast(&req.request);
        assert_eq!(result, vec!["btcusdt", "ethusdt", "ltcbtc", "bnbusdt"])
    }

    #[test]
    fn eval_ast() {
        let big = |n| BigDecimal::try_from(n).unwrap();

        let mut btcusdt = KlineResponse::default();
        btcusdt.data.k.open = big(5.9);
        btcusdt.data.k.high = big(10.3);
        btcusdt.data.k.low = big(5.9);
        btcusdt.data.k.close = big(7.1);

        let mut ethusdt = KlineResponse::default();
        ethusdt.data.k.open = big(105.0);
        ethusdt.data.k.high = big(107.3);
        ethusdt.data.k.low = big(102.1);
        ethusdt.data.k.close = big(104.6);

        let mut bnbusdt = KlineResponse::default();
        bnbusdt.data.k.open = big(50.1);
        bnbusdt.data.k.high = big(51.3);
        bnbusdt.data.k.low = big(50.0);
        bnbusdt.data.k.close = big(52.4);

        let vals = HashMap::from([
            ("btcusdt".to_string(), btcusdt),
            ("ethusdt".to_string(), ethusdt),
            ("bnbusdt".to_string(), bnbusdt),
        ]);

        let req = parse_stream_request("btcusdt+(ethusdt-bnbusdt)@1m").unwrap();
        let result = eval_ast_with(&req.request, &vals).unwrap();

        assert_eq!(result.data.k.open, big(60.8));
        assert_eq!(result.data.k.high, big(66.3));
        assert_eq!(result.data.k.low, big(58.0));
        assert_eq!(result.data.k.close, big(59.3));
    }
}
