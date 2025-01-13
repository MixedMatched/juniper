use anyhow::Result;
use egg::{AstSize, Extractor, Id, RecExpr, Runner};
use juniper_lib::{approximate, get_juniper_rules, is_atomic, ConstantFold, MathExpression};
use std::io;

fn main() -> Result<()> {
    let rules = get_juniper_rules()?;

    loop {
        println!("Enter a (lisp-y) expression: ");
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let expr: RecExpr<MathExpression> = input.parse()?;

                let runner: Runner<MathExpression, ConstantFold> =
                    Runner::default().with_expr(&expr).run(&rules);
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
