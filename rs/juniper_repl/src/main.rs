use anyhow::{Error, Result};
use egg::{AstSize, Extractor, Id, Pattern, RecExpr, Rewrite, Runner};
use juniper_lib::{approximate, get_juniper_rules, is_atomic, ConstantFold, MathExpression};
use std::io;

fn get_complete_idx(str: &String) -> Result<usize> {
    Err(Error::msg("unimplemented"))
}

fn get_children(
    expr: &RecExpr<MathExpression>,
) -> Result<(RecExpr<MathExpression>, RecExpr<MathExpression>)> {
    let mut assign_string = format!("{expr}");
    assign_string.pop();
    let coupled_string = assign_string.split_off(3);
    let (s1, s2) = coupled_string.split_at(get_complete_idx(&coupled_string)?);

    Ok((s1.parse()?, s2.parse()?))
}

fn create_assignment(
    num: usize,
    expr: &RecExpr<MathExpression>,
) -> Result<Option<[Rewrite<MathExpression, ConstantFold>; 2]>> {
    match expr[expr.root()] {
        MathExpression::Assign(_) => {
            let (side_a, side_b) = get_children(expr)?;

            let pattern_a: Pattern<MathExpression> = side_a.into();
            let pattern_b: Pattern<MathExpression> = side_b.into();

            let name_forward = format!("assignment_{num}_f");
            let name_backward = format!("assignment_{num}_b");

            if let Ok(rewrite_forward) =
                Rewrite::new(name_forward, pattern_a.clone(), pattern_b.clone())
            {
                if let Ok(rewrite_backward) = Rewrite::new(name_backward, pattern_b, pattern_a) {
                    Ok(Some([rewrite_forward, rewrite_backward]))
                } else {
                    Err(Error::msg(
                        "backward rewrite construction for assignment failed",
                    ))
                }
            } else {
                Err(Error::msg(
                    "forward rewrite construction for assignment failed",
                ))
            }
        }
        _ => Ok(None),
    }
}

fn main() -> Result<()> {
    let mut rules = get_juniper_rules()?;
    let mut conditions = Vec::new();

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

                if let Ok(Some(rewrites)) = create_assignment(conditions.len(), &expr) {
                    conditions.extend_from_slice(&rewrites);
                    rules.extend_from_slice(&rewrites);
                }
            }
            Err(error) => println!("error: {error}"),
        }
    }
}
