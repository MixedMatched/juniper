use anyhow::{Error, Result};
use egg::{AstSize, Extractor, Id, Language, Pattern, RecExpr, Rewrite};
use juniper_lib::{
    approximate, get_juniper_rules, is_atomic, JuniperRewrite, JuniperRunner, MathExpression,
};
use std::{fs::File, io::{self, Write}};

// courtesy of Remy Wang on the E-Graphs Zulip
fn split<L: Language>(e: &RecExpr<L>) -> Vec<RecExpr<L>> {
    e[e.root()]
        .children()
        .iter()
        .map(|c| L::build_recexpr(&e[*c], |j: Id| e[j].clone()))
        .collect()
}

// creates a set of rewrites that will operate on either side of an assignment to the other side
fn create_assignment(
    num: usize,
    expr: &RecExpr<MathExpression>,
) -> Result<Option<[JuniperRewrite; 2]>> {
    match expr[expr.root()] {
        MathExpression::Assign(_) => {
            let children = split(expr);

            if let Some(side_a) = children.get(0) {
                if let Some(side_b) = children.get(1) {
                    let pattern_a: Pattern<MathExpression> = side_a.into();
                    let pattern_b: Pattern<MathExpression> = side_b.into();

                    let name_forward = format!("assignment_{num}_f");
                    let name_backward = format!("assignment_{num}_b");

                    if let Ok(rewrite_forward) =
                        Rewrite::new(name_forward, pattern_a.clone(), pattern_b.clone())
                    {
                        if let Ok(rewrite_backward) =
                            Rewrite::new(name_backward, pattern_b, pattern_a)
                        {
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
                } else {
                    Err(Error::msg("problem with parsing right side of assignment"))
                }
            } else {
                Err(Error::msg("problem with parsing left side of assignment"))
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

                let mut runner = JuniperRunner::default().with_explanations_enabled().with_expr(&expr).run(&rules);
                let extractor = Extractor::new(&runner.egraph, AstSize);

                let (_, best_expr) = extractor.find_best(runner.roots[0]);
                println!("{}", best_expr);

                let mut file = File::create("explanation.txt")?;
                let explanation = runner.explain_equivalence(&expr, &best_expr);
                file.write_all(explanation.get_string().as_bytes())?;

                if !is_atomic(&best_expr, &best_expr.root()) {
                    if let Some(approximation) = approximate(&best_expr, &best_expr.root()) {
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
