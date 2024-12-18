use anyhow::Error;
use anyhow::Result;

use egg::ENodeOrVar;
use egg::Pattern;
use egg::RecExpr;
use egg::Rewrite;
use juniper_math_expression::ConstantFold;
use juniper_math_expression::MathExpression;
use lean_parse::lean_expr::{LeanExpr, Name};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
enum LMEIntermediateConst {
    Rational(),
}

// an intermediate representation between conversion from Lean.Expr and MathExpression,
// which is partially instantiable
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
enum LMEIntermediateRep {
    Const(LMEIntermediateConst),
    Forall {
        binder_name: Option<Name>,
        binder_type: Option<Box<LMEIntermediateRep>>,
        body: Option<Box<LMEIntermediateRep>>,
    },
    Eq {
        all_type: Option<Name>,
        in1: Option<Box<LMEIntermediateRep>>,
        in2: Option<Box<LMEIntermediateRep>>,
    },
    Add {
        in1_type: Option<Name>,
        in2_type: Option<Name>,
        out_type: Option<Name>,
        in1: Option<Box<LMEIntermediateRep>>,
        in2: Option<Box<LMEIntermediateRep>>,
    },
}

impl LMEIntermediateRep {
    fn from_lean(expr: LeanExpr) -> Result<LMEIntermediateRep> {
        todo!()
    }

    fn split_at_top_eq(&self) -> Option<(LMEIntermediateRep, LMEIntermediateRep)> {
        todo!()
    }

    fn to_math_expression(self) -> Result<RecExpr<ENodeOrVar<MathExpression>>> {
        todo!()
    }
}

fn lean_to_rewrites(
    lean_exprs: Vec<(Name, LeanExpr)>,
) -> Result<Vec<Rewrite<MathExpression, ConstantFold>>> {
    let mut result = Vec::new();
    for (name, expr) in lean_exprs {
        let intermediate = LMEIntermediateRep::from_lean(expr)?;
        if let Some((eq1, eq2)) = intermediate.split_at_top_eq() {
            result.push(
                Rewrite::new(
                    name,
                    Pattern::new(eq1.to_math_expression()?),
                    Pattern::new(eq2.to_math_expression()?),
                )
                .expect("bad rewrite"),
            );
        } else {
            return Err(Error::msg("error in some rewrite creation"));
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
}
