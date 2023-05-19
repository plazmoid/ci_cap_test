use std::collections::HashMap;

use crate::ws;
use anyhow::{anyhow, bail, Context, Result};
use arithmetic_parser::{
    grammars::{F32Grammar, Parse, Untyped},
    Expr, LocatedSpan, Statement,
};

pub type ParserAST<'a> = LocatedSpan<&'a str, Expr<'a, Untyped<F32Grammar>>>;

pub fn parse_stream_request(s: &str) -> Result<ws::StreamRequestAST<'static>> {
    let full_req = s.split('@').collect::<Vec<_>>();
    let req = full_req[0];
    let interval = full_req[1];
    let mut block = Untyped::<F32Grammar>::parse_statements(req).context("block")?;
    let expression = block
        .statements
        .pop()
        .ok_or(anyhow!(
            "failed to parse stream expression: no expressions found"
        ))?
        .extra;
    match expression {
        Statement::Expr(expr) => Ok(ws::StreamRequestAST {
            request: expr,
            candle_interval: interval.to_string(),
        }),
        _ => bail!("failed to parse stream expression: wrong format"),
    }
}

pub fn get_all_variables_from_ast(ast: &ParserAST<'_>) -> Vec<String> {
    let mut vertices = vec![];
    //ast.extra.
    //let mut visited_vertices = vec![];

    vertices
}

pub fn eval_ast_with(ast: &ParserAST<'_>, vals: &HashMap<String, ws::KlineResponse>) {
    todo!()
}
