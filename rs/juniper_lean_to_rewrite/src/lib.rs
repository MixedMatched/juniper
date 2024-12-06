use std::str::FromStr;

use egg::Applier;
use egg::Pattern;
use egg::Rewrite;
use egg::Searcher;
use juniper_math_expression::ConstantFold;
use juniper_math_expression::MathExpression;
use lean_parse::lean_expr::LeanExpr;

fn lean_to_searcher(
    expr: &LeanExpr,
) -> impl Searcher<MathExpression, ConstantFold> + Send + Sync + 'static {
    Pattern::new(String::from_str("(* 5 5)").unwrap().parse().unwrap())
}

fn lean_to_applier(
    expr: &LeanExpr,
) -> impl Applier<MathExpression, ConstantFold> + Send + Sync + 'static {
    Pattern::new(String::from_str("(* 5 5)").unwrap().parse().unwrap())
}

pub fn lean_to_rewrite(
    name: String,
    expr: &LeanExpr,
) -> Result<Rewrite<MathExpression, ConstantFold>, String> {
    Rewrite::new(name, lean_to_searcher(expr), lean_to_applier(expr))
}
#[cfg(test)]
mod tests {
    use super::*;
}
