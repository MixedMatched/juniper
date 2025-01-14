use egg::{merge_option, Analysis, DidMerge, EGraph, Id, Language, PatternAst};
use num::{
    traits::Pow,
    BigInt, BigRational, FromPrimitive,
};

use crate::{JuniperBigRational, MathExpression};

#[derive(Default, Clone)]
pub struct ConstantFold;

impl Analysis<MathExpression> for ConstantFold {
    type Data = Option<(JuniperBigRational, PatternAst<MathExpression>)>;

    fn make(
        egraph: &mut EGraph<MathExpression, ConstantFold>,
        enode: &MathExpression,
    ) -> Self::Data {
        let x = |i: &Id| egraph[*i].data.as_ref().map(|d| d.0.clone());
        Some(match enode {
            MathExpression::Constant(c) => (c.clone(), format!("{}", c).parse().unwrap()),
            MathExpression::Add([a, b]) => (
                JuniperBigRational(x(a)?.0 + x(b)?.0),
                format!("(+ {} {})", x(a)?, x(b)?).parse().unwrap(),
            ),
            MathExpression::Sub([a, b]) => (
                JuniperBigRational(x(a)?.0 - x(b)?.0),
                format!("(- {} {})", x(a)?, x(b)?).parse().unwrap(),
            ),
            MathExpression::Mul([a, b]) => (
                JuniperBigRational(x(a)?.0 * x(b)?.0),
                format!("(* {} {})", x(a)?, x(b)?).parse().unwrap(),
            ),
            MathExpression::Div([a, b])
                if x(b) != Some(JuniperBigRational(BigRational::new(0.into(), 1.into()))) =>
            {
                (
                    JuniperBigRational(x(a)?.0 / x(b)?.0),
                    format!("(/ {} {})", x(a)?, x(b)?).parse().unwrap(),
                )
            }
            MathExpression::Pow([a, b])
                if x(a) != Some(JuniperBigRational(BigRational::new(0.into(), 1.into()))) =>
            {
                let exponent = x(b)?.0;
                if exponent.denom() == &BigInt::from_i8(1)? {
                    (
                        JuniperBigRational(x(a)?.0.pow(exponent.numer())),
                        format!("(^ {} {})", x(a)?, x(b)?).parse().unwrap(),
                    )
                } else {
                    return None;
                }
            }
            MathExpression::Neg(a) => (
                JuniperBigRational(-x(a)?.0),
                format!("(- {})", x(a)?).parse().unwrap(),
            ),
            _ => return None,
        })
    }

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
        merge_option(to, from, |a, b| {
            assert_eq!(a.0, b.0, "Merged non-equal constants");
            DidMerge(false, false)
        })
    }

    fn modify(egraph: &mut EGraph<MathExpression, ConstantFold>, id: Id) {
        let data = egraph[id].data.clone();
        if let Some((c, pat)) = data {
            if egraph.are_explanations_enabled() {
                egraph.union_instantiations(
                    &pat,
                    &format!("{}", c).parse().unwrap(),
                    &Default::default(),
                    "constant_fold".to_string(),
                );
            } else {
                let added = egraph.add(MathExpression::Constant(c));
                egraph.union(id, added);
            }
            // to not prune, comment this out
            egraph[id].nodes.retain(|n| n.is_leaf());

            #[cfg(debug_assertions)]
            egraph[id].assert_unique_leaves();
        }
    }
}
