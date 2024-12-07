use anyhow::Error;
use anyhow::Result;

use egg::Pattern;
use egg::Rewrite;
use juniper_math_expression::ConstantFold;
use juniper_math_expression::MathExpression;
use lean_parse::lean_expr::{LeanExpr, Name};

// get the children of the first instance of a single application with the given declName
fn expr_split_once(expr: &LeanExpr, decl_name: Name) -> Result<(LeanExpr, LeanExpr)> {
    todo!()
}

fn unwrap_forall_let(expr: &LeanExpr) -> Result<(Vec<Name>, LeanExpr)> {
    match expr {
        LeanExpr::ForallE {
            binder_name,
            binder_type,
            body,
            binder_info,
        } => todo!(),
        LeanExpr::LetE {
            decl_name,
            typ,
            value,
            body,
            non_dep,
        } => todo!(),
        _ => Err(Error::msg("no forall or let found")),
    }
}

fn expr_to_pattern(expr: &LeanExpr) -> Result<Pattern<MathExpression>> {
    todo!()
}

pub fn eq_to_rewrite(
    name: String,
    expr: &LeanExpr,
) -> Result<[Rewrite<MathExpression, ConstantFold>; 2]> {
    let lean_equality = expr_split_once(expr, "Eq".to_string())?;
    let pattern_equality = (
        expr_to_pattern(&lean_equality.0)?,
        expr_to_pattern(&lean_equality.1)?,
    );
    Ok([
        Rewrite::new(
            name.clone(),
            pattern_equality.0.clone(),
            pattern_equality.1.clone(),
        )
        .expect("bad 1st rewrite"),
        Rewrite::new(name, pattern_equality.1, pattern_equality.0).expect("bad 2nd rewrite"),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
}
