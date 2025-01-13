use anyhow::Result;
use egg::{AstSize, Id, RecExpr, Rewrite, Runner};
use juniper_lean_to_rewrite::JuniperJsonEntry;
pub use juniper_math_expression::{approximate, ConstantFold, MathExpression};

pub type JuniperRunner = Runner<MathExpression, ConstantFold>;
pub type JuniperRewrite = Rewrite<MathExpression, ConstantFold>;
pub type JuniperCostFunction = AstSize;

pub fn get_juniper_rules() -> Result<Vec<JuniperRewrite>> {
    let lean_theorems: Vec<JuniperJsonEntry> =
        serde_json::from_str(include_str!("../../../exported.json"))?;
    Ok(juniper_lean_to_rewrite::lean_to_rewrites(lean_theorems)?)
}

pub fn is_atomic(re: &RecExpr<MathExpression>, id: &Id) -> bool {
    match &re[*id] {
        MathExpression::Constant(_) => true,
        MathExpression::Variable(_) => true,
        _ => false,
    }
}
