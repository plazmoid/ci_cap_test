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
    vals: &HashMap<String, ws::ClientResponse>,
) -> Option<ws::ClientResponse> {
    let mut result: Option<ws::ClientResponse> = None;

    match &node.extra {
        Expr::Variable => {
            let pair = node.fragment().to_string();
            result = vals.get(&pair).cloned();
        }
        Expr::Binary { lhs, op, rhs } => {
            let lhs = eval_ast_with(&*lhs, vals)?;
            let rhs = eval_ast_with(&*rhs, vals)?;

            let mut kline_result = ws::ClientResponse::default();
            kline_result.data.kline_start_time =
                u64::max(lhs.data.kline_start_time, rhs.data.kline_start_time);
            match op.extra {
                BinaryOp::Add => {
                    kline_result.data.open = lhs.data.open + rhs.data.open;
                    kline_result.data.high = lhs.data.high + rhs.data.high;
                    kline_result.data.low = lhs.data.low + rhs.data.low;
                    kline_result.data.close = lhs.data.close + rhs.data.close;
                }
                BinaryOp::Sub => {
                    kline_result.data.open = lhs.data.open - rhs.data.open;
                    kline_result.data.high = lhs.data.high - rhs.data.high;
                    kline_result.data.low = lhs.data.low - rhs.data.low;
                    kline_result.data.close = lhs.data.close - rhs.data.close;
                }
                BinaryOp::Mul => {
                    kline_result.data.open = lhs.data.open * rhs.data.open;
                    kline_result.data.high = lhs.data.high * rhs.data.high;
                    kline_result.data.low = lhs.data.low * rhs.data.low;
                    kline_result.data.close = lhs.data.close * rhs.data.close;
                }
                BinaryOp::Div => {
                    // todo /0
                    kline_result.data.open = lhs.data.open / rhs.data.open;
                    kline_result.data.high = lhs.data.high / rhs.data.high;
                    kline_result.data.low = lhs.data.low / rhs.data.low;
                    kline_result.data.close = lhs.data.close / rhs.data.close;
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
    use crate::models::ws::ClientResponse;
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

        let mut btcusdt = ClientResponse::default();
        btcusdt.data.open = big(5.9);
        btcusdt.data.high = big(10.3);
        btcusdt.data.low = big(5.9);
        btcusdt.data.close = big(7.1);

        let mut ethusdt = ClientResponse::default();
        ethusdt.data.open = big(105.0);
        ethusdt.data.high = big(107.3);
        ethusdt.data.low = big(102.1);
        ethusdt.data.close = big(104.6);

        let mut bnbusdt = ClientResponse::default();
        bnbusdt.data.open = big(50.1);
        bnbusdt.data.high = big(51.3);
        bnbusdt.data.low = big(50.0);
        bnbusdt.data.close = big(52.4);

        let vals = HashMap::from([
            ("btcusdt".to_string(), btcusdt),
            ("ethusdt".to_string(), ethusdt),
            ("bnbusdt".to_string(), bnbusdt),
        ]);

        let req = parse_stream_request("btcusdt+(ethusdt-bnbusdt)@1m").unwrap();
        let result = eval_ast_with(&req.request, &vals).unwrap();

        assert_eq!(result.data.open, big(60.8));
        assert_eq!(result.data.high, big(66.3));
        assert_eq!(result.data.low, big(58.0));
        assert_eq!(result.data.close, big(59.3));
    }
}
