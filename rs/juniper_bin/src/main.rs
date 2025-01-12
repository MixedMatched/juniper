use anyhow::Result;
use egg::{AstSize, Extractor, Id, RecExpr, Rewrite, Runner};
use juniper_lean_to_rewrite::JuniperJsonEntry;
use juniper_math_expression::{approximate, ConstantFold, MathExpression};
use std::io;

fn is_atomic(re: &RecExpr<MathExpression>, id: &Id) -> bool {
    match &re[*id] {
        MathExpression::Constant(_) => true,
        MathExpression::Variable(_) => true,
        _ => false,
    }
}

fn main() -> Result<()> {
    let lean_theorems: Vec<JuniperJsonEntry> =
        serde_json::from_str(include_str!("../../../exported.json"))?;
    let rules: &[Rewrite<MathExpression, ConstantFold>] =
        &juniper_lean_to_rewrite::lean_to_rewrites(lean_theorems)?;

    loop {
        println!("Enter a (lisp-y) expression: ");
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let expr: RecExpr<MathExpression> = input.parse()?;

                let runner: Runner<MathExpression, ConstantFold> =
                    Runner::default().with_expr(&expr).run(rules);
                let extractor = Extractor::new(&runner.egraph, AstSize);

                let (_, best_expr) = extractor.find_best(runner.roots[0]);
                println!("{}", best_expr);

                let last_id = Id::from(best_expr.as_ref().len() - 1);
                if !is_atomic(&best_expr, &last_id) {
                    if let Some(approximation) = approximate(&best_expr, &last_id) {
                        println!("≈ {}", approximation);
                    }
                }
            }
            Err(error) => println!("error: {error}"),
        }
    }
}
