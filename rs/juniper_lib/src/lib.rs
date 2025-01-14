use anyhow::Result;
use egg::{Id, RecExpr, Rewrite, Runner};
use juniper_lean_to_rewrite::JuniperJsonEntry;
pub use juniper_math_expression::{approximate, ConstantFold, MathExpression};

pub type JuniperRunner = Runner<MathExpression, ConstantFold>;
pub type JuniperRewrite = Rewrite<MathExpression, ConstantFold>;

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

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use egg::{AstSize, Extractor};

    use crate::{get_juniper_rules, JuniperRunner};

    #[test]
    fn test_default_rules() -> Result<()> {
        let rules = get_juniper_rules()?;

        let neg_add_cancel = "(+ (- a) a)".parse()?;
        let neg_add_cancel_runner = JuniperRunner::default()
            .with_expr(&neg_add_cancel)
            .run(&rules);

        let neg_add_cancel_extractor = Extractor::new(&neg_add_cancel_runner.egraph, AstSize);
        let (_, neg_add_cancel_extracted) =
            neg_add_cancel_extractor.find_best(neg_add_cancel_runner.roots[0]);

        let neg_add_cancel_manual = "0".parse()?;

        assert_eq!(neg_add_cancel_extracted, neg_add_cancel_manual);

        let sin_sq_cos_sq = "(+ (^ (sin x) 2) (^ (cos x) 2))".parse()?;
        let sin_sq_cos_sq_runner = JuniperRunner::default()
            .with_expr(&sin_sq_cos_sq)
            .run(&rules);

        let sin_sq_cos_sq_extractor = Extractor::new(&sin_sq_cos_sq_runner.egraph, AstSize);
        let (_, sin_sq_cos_sq_extracted) =
            sin_sq_cos_sq_extractor.find_best(sin_sq_cos_sq_runner.roots[0]);

        let sin_sq_cos_sq_manual = "1".parse()?;

        assert_eq!(sin_sq_cos_sq_extracted, sin_sq_cos_sq_manual);

        Ok(())
    }
}
