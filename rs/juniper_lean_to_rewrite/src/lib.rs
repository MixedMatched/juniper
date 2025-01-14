use std::fmt::Display;

use anyhow::{Error, Result};

use egg::{Condition, ConditionEqual, ConditionalApplier, EGraph, Id, Pattern, Rewrite, Subst};
use juniper_math_expression::{ConstantFold, MathExpression};
use lean_parse::lean_expr::{LeanExpr, Literal, Name};
use num::BigInt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct JuniperJsonEntry {
    name: Name,
    #[serde(rename = "type")]
    typ: LeanExpr,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
struct Hole;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
enum LMEIntermediateDefinedConst {
    Pi,
}

impl Display for LMEIntermediateDefinedConst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pi => write!(f, "π"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
enum LMEIntermediateConst {
    OfNat {
        out_type: Option<Name>,
        val: Option<BigInt>,
        inst: Option<Hole>,
    },
    OfScientific {
        out_type: Option<Name>,
        inst: Option<Hole>,
        mantissa: Option<BigInt>,
        exponent_sign: Option<bool>,
        decimal_exponent: Option<usize>,
    },
}

impl Display for LMEIntermediateConst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OfNat { val, .. } => {
                if let Some(val) = val {
                    write!(f, "{val}")
                } else {
                    write!(f, "")
                }
            }
            Self::OfScientific {
                mantissa,
                exponent_sign,
                decimal_exponent,
                ..
            } => {
                if let Some(mantissa) = mantissa {
                    if let Some(exponent_sign) = exponent_sign {
                        if let Some(decimal_exponent) = decimal_exponent {
                            if *exponent_sign {
                                let mut mantissa_string = format!("{mantissa}");
                                mantissa_string
                                    .insert(mantissa_string.len() - decimal_exponent, '.');

                                write!(f, "{mantissa_string}")
                            } else {
                                write!(f, "{mantissa}e{decimal_exponent}")
                            }
                        } else {
                            write!(f, "")
                        }
                    } else {
                        write!(f, "")
                    }
                } else {
                    write!(f, "")
                }
            }
        }
    }
}

// an intermediate representation for conversion from Lean.Expr and MathExpression,
// which is partially instantiable
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
enum LMEIntermediateRep {
    DefinedConst(LMEIntermediateDefinedConst),
    Const(LMEIntermediateConst),
    Var(Name),
    Forall {
        binder_name: Option<Name>,
        binder_type: Option<Box<Self>>,
        body: Option<Box<Self>>,
    },
    Eq {
        all_type: Option<Name>,
        in1: Option<Box<Self>>,
        in2: Option<Box<Self>>,
    },
    Ne {
        all_type: Option<Name>,
        in1: Option<Box<Self>>,
        in2: Option<Box<Self>>,
    },
    HBool {
        operator: Option<String>,
        in1_type: Option<Name>,
        in2_type: Option<Name>,
        out_type: Option<Name>,
        inst: Option<Hole>,
        in1: Option<Box<Self>>,
        in2: Option<Box<Self>>,
    },
    TUnary {
        operator: Option<String>,
        all_type: Option<Name>,
        inst: Option<Hole>,
        in1: Option<Box<Self>>,
    },
    IUnary {
        operator: Option<String>,
        in1: Option<Box<Self>>,
    },
}

impl Display for LMEIntermediateRep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DefinedConst(dc) => write!(f, "{dc}"),
            Self::Const(c) => write!(f, "{c}"),
            Self::Var(name) => write!(f, "?{name}"),
            Self::Forall { body, .. } => {
                if let Some(body) = body {
                    write!(f, "{body}")
                } else {
                    write!(f, "")
                }
            }
            Self::Eq { in1, in2, .. } => {
                if let Some(in1) = in1 {
                    if let Some(in2) = in2 {
                        write!(f, "(= {in1} {in2})")
                    } else {
                        write!(f, "")
                    }
                } else {
                    write!(f, "")
                }
            }
            Self::Ne { in1, in2, .. } => {
                if let Some(in1) = in1 {
                    if let Some(in2) = in2 {
                        write!(f, "(≠ {in1} {in2})")
                    } else {
                        write!(f, "")
                    }
                } else {
                    write!(f, "")
                }
            }
            Self::HBool {
                operator, in1, in2, ..
            } => {
                if let Some(in1) = in1 {
                    if let Some(in2) = in2 {
                        if let Some(operator) = operator {
                            write!(f, "({operator} {in1} {in2})")
                        } else {
                            write!(f, "")
                        }
                    } else {
                        write!(f, "")
                    }
                } else {
                    write!(f, "")
                }
            }
            Self::TUnary { operator, in1, .. } | Self::IUnary { operator, in1 } => {
                if let Some(in1) = in1 {
                    if let Some(operator) = operator {
                        write!(f, "({operator} {in1})")
                    } else {
                        write!(f, "")
                    }
                } else {
                    write!(f, "")
                }
            }
        }
    }
}

impl LMEIntermediateRep {
    // convert a declName to an uninstantiated intermediate representation
    fn name_to_ir(name: Name) -> Result<Self> {
        match name.as_str() {
            "Real.pi" => Ok(Self::DefinedConst(LMEIntermediateDefinedConst::Pi)),
            "OfScientific.ofScientific" => Ok(Self::Const(LMEIntermediateConst::OfScientific {
                out_type: None,
                inst: None,
                mantissa: None,
                exponent_sign: None,
                decimal_exponent: None,
            })),
            "OfNat.ofNat" => Ok(Self::Const(LMEIntermediateConst::OfNat {
                out_type: None,
                val: None,
                inst: None,
            })),
            "Eq" => Ok(Self::Eq {
                all_type: None,
                in1: None,
                in2: None,
            }),
            "Ne" => Ok(Self::Ne {
                all_type: None,
                in1: None,
                in2: None,
            }),
            "HAdd.hAdd" => Ok(Self::HBool {
                operator: Some("+".to_string()),
                in1_type: None,
                in2_type: None,
                out_type: None,
                inst: None,
                in1: None,
                in2: None,
            }),
            "HSub.hSub" => Ok(Self::HBool {
                operator: Some("-".to_string()),
                in1_type: None,
                in2_type: None,
                out_type: None,
                inst: None,
                in1: None,
                in2: None,
            }),
            "HMul.hMul" => Ok(Self::HBool {
                operator: Some("*".to_string()),
                in1_type: None,
                in2_type: None,
                out_type: None,
                inst: None,
                in1: None,
                in2: None,
            }),
            "HDiv.hDiv" => Ok(Self::HBool {
                operator: Some("/".to_string()),
                in1_type: None,
                in2_type: None,
                out_type: None,
                inst: None,
                in1: None,
                in2: None,
            }),
            "HPow.hPow" => Ok(Self::HBool {
                operator: Some("^".to_string()),
                in1_type: None,
                in2_type: None,
                out_type: None,
                inst: None,
                in1: None,
                in2: None,
            }),
            "Neg.neg" => Ok(Self::TUnary {
                operator: Some("-".to_string()),
                all_type: None,
                inst: None,
                in1: None,
            }),
            "Inv.inv" => Ok(Self::TUnary {
                operator: Some("inv".to_string()),
                all_type: None,
                inst: None,
                in1: None,
            }),
            "Real.sin" => Ok(Self::IUnary {
                operator: Some("sin".to_string()),
                in1: None,
            }),
            "Real.cos" => Ok(Self::IUnary {
                operator: Some("cos".to_string()),
                in1: None,
            }),
            "Real.sqrt" => Ok(Self::IUnary {
                operator: Some("sqrt".to_string()),
                in1: None,
            }),
            _ => Err(Error::msg(format!("unknown name: {}", name))),
        }
    }

    // parse the type components of LeanExprs into Names
    fn type_parse(arg: LeanExpr, _de_bruijn_names: Vec<Name>) -> Result<Name> {
        match arg {
            LeanExpr::Const { decl_name, .. } => Ok(decl_name),
            _ => Err(Error::msg(format!("bad type: {}", arg))),
        }
    }

    // transitions the partial instantiation to include the next apply argument
    fn app_state_next(current: Self, arg: LeanExpr, de_bruijn_names: Vec<Name>) -> Result<Self> {
        // because the arguments are ordered, we only have to specify that one argument is None
        // also this is really ugly, but that's mostly bc rust enum structs lack default support lol
        Ok(match current {
            Self::Const(LMEIntermediateConst::OfNat { out_type: None, .. }) => {
                Self::Const(LMEIntermediateConst::OfNat {
                    out_type: Some(Self::type_parse(arg, de_bruijn_names.clone())?),
                    val: None,
                    inst: None,
                })
            }
            Self::Const(LMEIntermediateConst::OfNat {
                out_type,
                val: None,
                ..
            }) => Self::Const(LMEIntermediateConst::OfNat {
                out_type,
                val: match arg {
                    LeanExpr::Lit(Literal::NatVal { val }) => Some(val.into()),
                    _ => return Err(Error::msg(format!("bad OfNat val: {}", arg))),
                },
                inst: None,
            }),
            Self::Const(LMEIntermediateConst::OfNat {
                out_type,
                val,
                inst: None,
            }) => Self::Const(LMEIntermediateConst::OfNat {
                out_type,
                val,
                inst: Some(Hole),
            }),
            Self::Const(LMEIntermediateConst::OfScientific { out_type: None, .. }) => {
                Self::Const(LMEIntermediateConst::OfScientific {
                    out_type: Some(Self::type_parse(arg, de_bruijn_names.clone())?),
                    inst: None,
                    mantissa: None,
                    exponent_sign: None,
                    decimal_exponent: None,
                })
            }
            Self::Const(LMEIntermediateConst::OfScientific {
                out_type,
                inst: None,
                ..
            }) => Self::Const(LMEIntermediateConst::OfScientific {
                out_type,
                inst: Some(Hole),
                mantissa: None,
                exponent_sign: None,
                decimal_exponent: None,
            }),
            Self::Const(LMEIntermediateConst::OfScientific {
                out_type,
                inst,
                mantissa: None,
                ..
            }) => Self::Const(LMEIntermediateConst::OfScientific {
                out_type,
                inst,
                mantissa: match arg {
                    LeanExpr::Lit(Literal::NatVal { val }) => Some(val.into()),
                    _ => return Err(Error::msg(format!("bad mantissa: {}", arg))),
                },
                exponent_sign: None,
                decimal_exponent: None,
            }),
            Self::Const(LMEIntermediateConst::OfScientific {
                out_type,
                inst,
                mantissa,
                exponent_sign: None,
                ..
            }) => Self::Const(LMEIntermediateConst::OfScientific {
                out_type,
                inst,
                mantissa,
                exponent_sign: match arg {
                    LeanExpr::Const { decl_name, .. } => match decl_name.as_str() {
                        "Bool.true" => Some(true),
                        "Bool.false" => Some(false),
                        _ => {
                            return Err(Error::msg(format!(
                                "bad exponent_sign const: {}",
                                decl_name
                            )))
                        }
                    },
                    _ => return Err(Error::msg(format!("bad exponent_sign arg: {}", arg))),
                },
                decimal_exponent: None,
            }),
            Self::Const(LMEIntermediateConst::OfScientific {
                out_type,
                inst,
                mantissa,
                exponent_sign,
                decimal_exponent: None,
            }) => Self::Const(LMEIntermediateConst::OfScientific {
                out_type,
                inst,
                mantissa,
                exponent_sign,
                decimal_exponent: match arg {
                    LeanExpr::Lit(Literal::NatVal { val }) => Some(val.try_into()?),
                    _ => return Err(Error::msg(format!("bad decimal_exponent: {}", arg))),
                },
            }),
            Self::Eq { all_type: None, .. } => Self::Eq {
                all_type: Some(Self::type_parse(arg, de_bruijn_names.clone())?),
                in1: None,
                in2: None,
            },
            Self::Eq {
                all_type,
                in1: None,
                ..
            } => Self::Eq {
                all_type,
                in1: Some(Box::new(Self::from_lean_recursive(
                    arg,
                    de_bruijn_names.clone(),
                )?)),
                in2: None,
            },
            Self::Eq {
                all_type,
                in1,
                in2: None,
            } => Self::Eq {
                all_type,
                in1,
                in2: Some(Box::new(Self::from_lean_recursive(
                    arg,
                    de_bruijn_names.clone(),
                )?)),
            },
            Self::Ne { all_type: None, .. } => Self::Ne {
                all_type: Some(Self::type_parse(arg, de_bruijn_names.clone())?),
                in1: None,
                in2: None,
            },
            Self::Ne {
                all_type,
                in1: None,
                ..
            } => Self::Ne {
                all_type,
                in1: Some(Box::new(Self::from_lean_recursive(
                    arg,
                    de_bruijn_names.clone(),
                )?)),
                in2: None,
            },
            Self::Ne {
                all_type,
                in1,
                in2: None,
            } => Self::Ne {
                all_type,
                in1,
                in2: Some(Box::new(Self::from_lean_recursive(
                    arg,
                    de_bruijn_names.clone(),
                )?)),
            },
            Self::HBool {
                operator,
                in1_type: None,
                ..
            } => Self::HBool {
                operator,
                in1_type: Some(Self::type_parse(arg, de_bruijn_names.clone())?),
                in2_type: None,
                out_type: None,
                inst: None,
                in1: None,
                in2: None,
            },
            Self::HBool {
                operator,
                in1_type,
                in2_type: None,
                ..
            } => Self::HBool {
                operator,
                in1_type,
                in2_type: Some(Self::type_parse(arg, de_bruijn_names.clone())?),
                out_type: None,
                inst: None,
                in1: None,
                in2: None,
            },
            Self::HBool {
                operator,
                in1_type,
                in2_type,
                out_type: None,
                ..
            } => Self::HBool {
                operator,
                in1_type,
                in2_type,
                out_type: Some(Self::type_parse(arg, de_bruijn_names.clone())?),
                inst: None,
                in1: None,
                in2: None,
            },
            Self::HBool {
                operator,
                in1_type,
                in2_type,
                out_type,
                inst: None,
                ..
            } => Self::HBool {
                operator,
                in1_type,
                in2_type,
                out_type,
                inst: Some(Hole),
                in1: None,
                in2: None,
            },
            Self::HBool {
                operator,
                in1_type,
                in2_type,
                out_type,
                inst,
                in1: None,
                ..
            } => Self::HBool {
                operator,
                in1_type,
                in2_type,
                out_type,
                inst,
                in1: Some(Box::new(Self::from_lean_recursive(
                    arg,
                    de_bruijn_names.clone(),
                )?)),
                in2: None,
            },
            Self::HBool {
                operator,
                in1_type,
                in2_type,
                out_type,
                inst,
                in1,
                in2: None,
            } => Self::HBool {
                operator,
                in1_type,
                in2_type,
                out_type,
                inst,
                in1,
                in2: Some(Box::new(Self::from_lean_recursive(
                    arg,
                    de_bruijn_names.clone(),
                )?)),
            },
            Self::TUnary {
                operator,
                all_type: None,
                ..
            } => Self::TUnary {
                operator,
                all_type: Some(Self::type_parse(arg, de_bruijn_names.clone())?),
                inst: None,
                in1: None,
            },
            Self::TUnary {
                operator,
                all_type,
                inst: None,
                ..
            } => Self::TUnary {
                operator,
                all_type,
                inst: Some(Hole),
                in1: None,
            },
            Self::TUnary {
                operator,
                all_type,
                inst,
                in1: None,
            } => Self::TUnary {
                operator,
                all_type,
                inst,
                in1: Some(Box::new(Self::from_lean_recursive(
                    arg,
                    de_bruijn_names.clone(),
                )?)),
            },
            Self::IUnary {
                operator,
                in1: None,
            } => Self::IUnary {
                operator,
                in1: Some(Box::new(Self::from_lean_recursive(
                    arg,
                    de_bruijn_names.clone(),
                )?)),
            },
            _ => {
                return Err(Error::msg(format!(
                    "unimplemented or already complete apply found: {:?} (with arg {})",
                    current, arg
                )))
            }
        })
    }

    // parses LeanExpr::App into an intermediate representation
    fn app_parse(
        function: Box<LeanExpr>,
        arg: Box<LeanExpr>,
        de_bruijn_names: Vec<Name>,
    ) -> Result<Self> {
        match *function {
            // recursive case: App
            LeanExpr::App {
                function: function_new,
                arg: arg_new,
            } => {
                let downstream = Self::app_parse(function_new, arg_new, de_bruijn_names.clone())?;
                Self::app_state_next(downstream, *arg, de_bruijn_names)
            }
            // base case: Const
            LeanExpr::Const { decl_name, .. } => {
                // we want to add the first argument to the new instance
                Self::app_state_next(Self::name_to_ir(decl_name)?, *arg, de_bruijn_names)
            }
            _ => Err(Error::msg(format!("unknown apply component: {}", function))),
        }
    }

    // recursively convert LeanExprs into intermediate representations
    fn from_lean_recursive(expr: LeanExpr, de_bruijn_names: Vec<Name>) -> Result<Self> {
        match expr {
            LeanExpr::ForallE {
                binder_name,
                binder_type,
                body,
                binder_info: _,
            } => Ok(Self::Forall {
                binder_name: Some(binder_name.clone()),
                binder_type: Some(Box::new(Self::from_lean_recursive(
                    *binder_type,
                    de_bruijn_names.clone(),
                )?)),
                body: Some(Box::new(Self::from_lean_recursive(*body, {
                    let mut new_names = de_bruijn_names.clone();
                    new_names.push(binder_name);
                    new_names
                })?)),
            }),
            LeanExpr::App { function, arg } => Self::app_parse(function, arg, de_bruijn_names),
            LeanExpr::BVar { de_bruijn_index } => Ok(Self::Var(
                if let Some(name) =
                    de_bruijn_names.get(de_bruijn_names.len() - 1 - de_bruijn_index as usize)
                {
                    name.clone()
                } else {
                    return Err(Error::msg(format!(
                        "bad deBruijn index {} for names: {:?}",
                        de_bruijn_index,
                        de_bruijn_names.clone()
                    )));
                },
            )),
            LeanExpr::Const { decl_name, .. } => {
                if let Ok(ir) = Self::name_to_ir(decl_name) {
                    Ok(ir)
                } else {
                    Ok(Self::DefinedConst(LMEIntermediateDefinedConst::Pi))
                }
            }
            _ => Err(Error::msg(format!(
                "improper top-level structure: {}",
                expr
            ))),
        }
    }

    // convert LeanExpr into an intermediate representation
    fn from_lean(expr: LeanExpr) -> Result<Self> {
        Self::from_lean_recursive(expr, Vec::new())
    }

    // split apart intermediate representations at their top-level Eq (collecting condition Foralls and ignoring others)
    fn split_at_top_eq(&self, conditions: Vec<Self>) -> Option<(Vec<Self>, Self, Self)> {
        match self {
            Self::Forall {
                body, binder_type, ..
            } => {
                if let Some(body) = body {
                    if let Some(binder_type) = binder_type {
                        match *binder_type.clone() {
                            Self::Const(_) | Self::DefinedConst(_) => {
                                body.split_at_top_eq(conditions)
                            }
                            b => body.split_at_top_eq({
                                let mut new_conditions = conditions.clone();
                                new_conditions.push(b);
                                new_conditions
                            }),
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Self::Eq { in1, in2, .. } => {
                if let Some(in1) = in1 {
                    if let Some(in2) = in2 {
                        Some((conditions, (**in1).clone(), (**in2).clone()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn as_condition(
        self,
    ) -> Result<
        Box<dyn Fn(&mut EGraph<MathExpression, ConstantFold>, Id, &Subst) -> bool + Send + Sync>,
    > {
        match self {
            Self::Eq { in1: Some(in1), in2: Some(in2), .. } => {
                let in1 = in1.to_math_expression()?;
                let in2 = in2.to_math_expression()?;

                let check_eq = ConditionEqual::new(in1, in2);

                Ok(Box::new(move |egraph, id, subst| {
                    check_eq.check(egraph, id, subst)
                }))
            },
            Self::Ne {  in1: Some(in1), in2: Some(in2), .. } => {
                let in1 = in1.to_math_expression()?;
                let in2 = in2.to_math_expression()?;

                let check_eq = ConditionEqual::new(in1, in2);

                // this is technically unsound (if, in the future, in1 = in2, just not when you're
                // currently checking)
                Ok(Box::new(move |egraph, id, subst| {
                    !check_eq.check(egraph, id, subst)
                }))
            }
            _ => Err(Error::msg(format!("Self::as_condition could not successfully convert the condition {self} into a closure"))),
        }
    }

    // convert intermediate representations into MathExpression Patterns
    fn to_math_expression(self) -> Result<Pattern<MathExpression>> {
        // this is really dumb, but I don't want to deal with RecExpr Ids
        Ok(format!("{self}").parse()?)
    }
}

// this is truly closure hell lol
fn create_condition_applier(
    applier: Pattern<MathExpression>,
    conditions: Vec<LMEIntermediateRep>,
) -> Result<
    ConditionalApplier<
        Box<dyn Fn(&mut EGraph<MathExpression, ConstantFold>, Id, &Subst) -> bool + Send + Sync>,
        Pattern<MathExpression>,
    >,
> {
    if !conditions.is_empty() { println!("creating cond app for {applier:?} with conditions {conditions:?}"); }
    Ok(ConditionalApplier {
        condition: Box::new({
            conditions.into_iter().try_rfold::<Box<
                dyn Fn(&mut EGraph<MathExpression, ConstantFold>, Id, &Subst) -> bool + Send + Sync,
            >, _, Result<_>>(
                Box::new(|_, _, _| true),
                |acc, condition| {
                    let condition_function = condition.clone().as_condition()?;
                    Ok(Box::new(move |e, i, s| {
                        acc(e, i, s) && condition_function(e, i, s)
                    }))
                },
            )?
        }),
        applier,
    })
}

// convert a list of named LeanExprs into egg MathExpression rewrite rules
pub fn lean_to_rewrites(
    lean_exprs: Vec<JuniperJsonEntry>,
) -> Result<Vec<Rewrite<MathExpression, ConstantFold>>> {
    let mut result = Vec::new();
    for JuniperJsonEntry { name, typ: expr } in lean_exprs {
        let intermediate = LMEIntermediateRep::from_lean(expr)?;
        if let Some((conditions, eq1, eq2)) = intermediate.split_at_top_eq(Vec::new()) {
            let eq1_me = eq1.to_math_expression()?;
            let eq2_me = eq2.to_math_expression()?;

            let forward_rewrite = Rewrite::new(
                name.clone() + "_forward",
                eq1_me.clone(),
                create_condition_applier(eq2_me.clone(), conditions.clone())?,
            );
            let backward_rewrite = Rewrite::new(
                name + "_backward",
                eq2_me,
                create_condition_applier(eq1_me, conditions)?,
            );

            // we only want to error out if no possible interpretation of the given theorem was
            // correct
            match (forward_rewrite, backward_rewrite) {
                (Err(ef), Err(eb)) => {
                    return Err(Error::msg(format!(
                        "forward and backward rewrite failed:\n\tforward: {ef}\n\tbackward: {eb}"
                    )))
                }
                (Ok(f), Err(_)) => result.push(f),
                (Err(_), Ok(b)) => result.push(b),
                (Ok(f), Ok(b)) => {
                    result.push(f);
                    result.push(b);
                }
            };
        } else {
            return Err(Error::msg(format!(
                "error in some rewrite creation for {name}"
            )));
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_from_lean() -> Result<()> {
        let add_zero_lean =
            serde_json::from_str::<LeanExpr>(include_str!("../../test_assets/add_zero.json"))
                .unwrap();
        let add_zero_ir = LMEIntermediateRep::from_lean(add_zero_lean)?;
        let add_zero_manual = LMEIntermediateRep::Forall {
            binder_name: Some("a".to_string()),
            binder_type: Some(Box::new(LMEIntermediateRep::DefinedConst(
                LMEIntermediateDefinedConst::Pi,
            ))),
            body: Some(Box::new(LMEIntermediateRep::Eq {
                all_type: Some("Rat".to_string()),
                in1: Some(Box::new(LMEIntermediateRep::HBool {
                    operator: Some("+".to_string()),
                    in1_type: Some("Rat".to_string()),
                    in2_type: Some("Rat".to_string()),
                    out_type: Some("Rat".to_string()),
                    inst: Some(Hole),
                    in1: Some(Box::new(LMEIntermediateRep::Var("a".to_string()))),
                    in2: Some(Box::new(LMEIntermediateRep::Const(
                        LMEIntermediateConst::OfNat {
                            out_type: Some("Rat".to_string()),
                            val: Some(0.into()),
                            inst: Some(Hole),
                        },
                    ))),
                })),
                in2: Some(Box::new(LMEIntermediateRep::Var("a".to_string()))),
            })),
        };

        assert_eq!(add_zero_ir, add_zero_manual);

        let cos_pi_div_four_lean = serde_json::from_str::<LeanExpr>(include_str!(
            "../../test_assets/cos_pi_div_four.json"
        ))
        .unwrap();
        let cos_pi_div_four_ir = LMEIntermediateRep::from_lean(cos_pi_div_four_lean)?;
        let cos_pi_div_four_manual = LMEIntermediateRep::Eq {
            all_type: Some("Real".to_string()),
            in1: Some(Box::new(LMEIntermediateRep::IUnary {
                operator: Some("cos".to_string()),
                in1: Some(Box::new(LMEIntermediateRep::HBool {
                    operator: Some("/".to_string()),
                    in1_type: Some("Real".to_string()),
                    in2_type: Some("Real".to_string()),
                    out_type: Some("Real".to_string()),
                    inst: Some(Hole),
                    in1: Some(Box::new(LMEIntermediateRep::DefinedConst(
                        LMEIntermediateDefinedConst::Pi,
                    ))),
                    in2: Some(Box::new(LMEIntermediateRep::Const(
                        LMEIntermediateConst::OfNat {
                            out_type: Some("Real".to_string()),
                            val: Some(4.into()),
                            inst: Some(Hole),
                        },
                    ))),
                })),
            })),
            in2: Some(Box::new(LMEIntermediateRep::HBool {
                operator: Some("/".to_string()),
                in1_type: Some("Real".to_string()),
                in2_type: Some("Real".to_string()),
                out_type: Some("Real".to_string()),
                inst: Some(Hole),
                in1: Some(Box::new(LMEIntermediateRep::IUnary {
                    operator: Some("sqrt".to_string()),
                    in1: Some(Box::new(LMEIntermediateRep::Const(
                        LMEIntermediateConst::OfNat {
                            out_type: Some("Real".to_string()),
                            val: Some(2.into()),
                            inst: Some(Hole),
                        },
                    ))),
                })),
                in2: Some(Box::new(LMEIntermediateRep::Const(
                    LMEIntermediateConst::OfNat {
                        out_type: Some("Real".to_string()),
                        val: Some(2.into()),
                        inst: Some(Hole),
                    },
                ))),
            })),
        };

        assert_eq!(cos_pi_div_four_ir, cos_pi_div_four_manual);

        let inv_neg_lean =
            serde_json::from_str::<LeanExpr>(include_str!("../../test_assets/inv_neg.json"))
                .unwrap();
        let inv_neg_ir = LMEIntermediateRep::from_lean(inv_neg_lean)?;
        let inv_neg_manual = LMEIntermediateRep::Forall {
            binder_name: Some("q".to_string()),
            binder_type: Some(Box::new(LMEIntermediateRep::DefinedConst(
                LMEIntermediateDefinedConst::Pi,
            ))),
            body: Some(Box::new(LMEIntermediateRep::Eq {
                all_type: Some("Rat".to_string()),
                in1: Some(Box::new(LMEIntermediateRep::TUnary {
                    operator: Some("inv".to_string()),
                    all_type: Some("Rat".to_string()),
                    inst: Some(Hole),
                    in1: Some(Box::new(LMEIntermediateRep::TUnary {
                        operator: Some("-".to_string()),
                        all_type: Some("Rat".to_string()),
                        inst: Some(Hole),
                        in1: Some(Box::new(LMEIntermediateRep::Var("q".to_string()))),
                    })),
                })),
                in2: Some(Box::new(LMEIntermediateRep::TUnary {
                    operator: Some("-".to_string()),
                    all_type: Some("Rat".to_string()),
                    inst: Some(Hole),
                    in1: Some(Box::new(LMEIntermediateRep::TUnary {
                        operator: Some("inv".to_string()),
                        all_type: Some("Rat".to_string()),
                        inst: Some(Hole),
                        in1: Some(Box::new(LMEIntermediateRep::Var("q".to_string()))),
                    })),
                })),
            })),
        };

        assert_eq!(inv_neg_ir, inv_neg_manual);

        let mul_comm_lean =
            serde_json::from_str::<LeanExpr>(include_str!("../../test_assets/mul_comm.json"))
                .unwrap();
        let mul_comm_ir = LMEIntermediateRep::from_lean(mul_comm_lean)?;
        let mul_comm_manual = LMEIntermediateRep::Forall {
            binder_name: Some("a".to_string()),
            binder_type: Some(Box::new(LMEIntermediateRep::DefinedConst(
                LMEIntermediateDefinedConst::Pi,
            ))),
            body: Some(Box::new(LMEIntermediateRep::Forall {
                binder_name: Some("b".to_string()),
                binder_type: Some(Box::new(LMEIntermediateRep::DefinedConst(
                    LMEIntermediateDefinedConst::Pi,
                ))),
                body: Some(Box::new(LMEIntermediateRep::Eq {
                    all_type: Some("Rat".to_string()),
                    in1: Some(Box::new(LMEIntermediateRep::HBool {
                        operator: Some("*".to_string()),
                        in1_type: Some("Rat".to_string()),
                        in2_type: Some("Rat".to_string()),
                        out_type: Some("Rat".to_string()),
                        inst: Some(Hole),
                        in1: Some(Box::new(LMEIntermediateRep::Var("a".to_string()))),
                        in2: Some(Box::new(LMEIntermediateRep::Var("b".to_string()))),
                    })),
                    in2: Some(Box::new(LMEIntermediateRep::HBool {
                        operator: Some("*".to_string()),
                        in1_type: Some("Rat".to_string()),
                        in2_type: Some("Rat".to_string()),
                        out_type: Some("Rat".to_string()),
                        inst: Some(Hole),
                        in1: Some(Box::new(LMEIntermediateRep::Var("b".to_string()))),
                        in2: Some(Box::new(LMEIntermediateRep::Var("a".to_string()))),
                    })),
                })),
            })),
        };

        assert_eq!(mul_comm_ir, mul_comm_manual);

        let mul_inv_cancel_lean =
            serde_json::from_str::<LeanExpr>(include_str!("../../test_assets/mul_inv_cancel.json"))
                .unwrap();
        let mul_inv_cancel_ir = LMEIntermediateRep::from_lean(mul_inv_cancel_lean)?;
        let mul_inv_cancel_manual = LMEIntermediateRep::Forall {
            binder_name: Some("a".to_string()),
            binder_type: Some(Box::new(LMEIntermediateRep::DefinedConst(
                LMEIntermediateDefinedConst::Pi,
            ))),
            body: Some(Box::new(LMEIntermediateRep::Forall {
                binder_name: Some("a._@.Mathlib.Data.Rat.Defs._hyg.3473".to_string()),
                binder_type: Some(Box::new(LMEIntermediateRep::Ne {
                    all_type: Some("Rat".to_string()),
                    in1: Some(Box::new(LMEIntermediateRep::Var("a".to_string()))),
                    in2: Some(Box::new(LMEIntermediateRep::Const(
                        LMEIntermediateConst::OfNat {
                            out_type: Some("Rat".to_string()),
                            val: Some(0.into()),
                            inst: Some(Hole),
                        },
                    ))),
                })),
                body: Some(Box::new(LMEIntermediateRep::Eq {
                    all_type: Some("Rat".to_string()),
                    in1: Some(Box::new(LMEIntermediateRep::HBool {
                        operator: Some("*".to_string()),
                        in1_type: Some("Rat".to_string()),
                        in2_type: Some("Rat".to_string()),
                        out_type: Some("Rat".to_string()),
                        inst: Some(Hole),
                        in1: Some(Box::new(LMEIntermediateRep::Var("a".to_string()))),
                        in2: Some(Box::new(LMEIntermediateRep::TUnary {
                            operator: Some("inv".to_string()),
                            all_type: Some("Rat".to_string()),
                            inst: Some(Hole),
                            in1: Some(Box::new(LMEIntermediateRep::Var("a".to_string()))),
                        })),
                    })),
                    in2: Some(Box::new(LMEIntermediateRep::Const(
                        LMEIntermediateConst::OfNat {
                            out_type: Some("Rat".to_string()),
                            val: Some(1.into()),
                            inst: Some(Hole),
                        },
                    ))),
                })),
            })),
        };

        assert_eq!(mul_inv_cancel_ir, mul_inv_cancel_manual);

        let neg_add_cancel_lean =
            serde_json::from_str::<LeanExpr>(include_str!("../../test_assets/neg_add_cancel.json"))
                .unwrap();
        let neg_add_cancel_ir = LMEIntermediateRep::from_lean(neg_add_cancel_lean)?;
        let neg_add_cancel_manual = LMEIntermediateRep::Forall {
            binder_name: Some("a".to_string()),
            binder_type: Some(Box::new(LMEIntermediateRep::DefinedConst(
                LMEIntermediateDefinedConst::Pi,
            ))),
            body: Some(Box::new(LMEIntermediateRep::Eq {
                all_type: Some("Rat".to_string()),
                in1: Some(Box::new(LMEIntermediateRep::HBool {
                    operator: Some("+".to_string()),
                    in1_type: Some("Rat".to_string()),
                    in2_type: Some("Rat".to_string()),
                    out_type: Some("Rat".to_string()),
                    inst: Some(Hole),
                    in1: Some(Box::new(LMEIntermediateRep::TUnary {
                        operator: Some("-".to_string()),
                        all_type: Some("Rat".to_string()),
                        inst: Some(Hole),
                        in1: Some(Box::new(LMEIntermediateRep::Var("a".to_string()))),
                    })),
                    in2: Some(Box::new(LMEIntermediateRep::Var("a".to_string()))),
                })),
                in2: Some(Box::new(LMEIntermediateRep::Const(
                    LMEIntermediateConst::OfNat {
                        out_type: Some("Rat".to_string()),
                        val: Some(0.into()),
                        inst: Some(Hole),
                    },
                ))),
            })),
        };

        assert_eq!(neg_add_cancel_ir, neg_add_cancel_manual);

        Ok(())
    }

    #[test]
    fn test_display() -> Result<()> {
        let add_zero_lean =
            serde_json::from_str::<LeanExpr>(include_str!("../../test_assets/add_zero.json"))
                .unwrap();
        let add_zero_ir = LMEIntermediateRep::from_lean(add_zero_lean)?;
        let add_zero_str = format!("{add_zero_ir}");
        let manual_add_zero = "(= (+ ?a 0) ?a)".to_string();

        assert_eq!(add_zero_str, manual_add_zero);

        let cos_pi_div_four_lean = serde_json::from_str::<LeanExpr>(include_str!(
            "../../test_assets/cos_pi_div_four.json"
        ))
        .unwrap();
        let cos_pi_div_four_ir = LMEIntermediateRep::from_lean(cos_pi_div_four_lean)?;
        let cos_pi_div_four_str = format!("{cos_pi_div_four_ir}");
        let manual_cos_pi_div_four = "(= (cos (/ π 4)) (/ (sqrt 2) 2))".to_string();

        assert_eq!(cos_pi_div_four_str, manual_cos_pi_div_four);

        let inv_neg_lean =
            serde_json::from_str::<LeanExpr>(include_str!("../../test_assets/inv_neg.json"))
                .unwrap();
        let inv_neg_ir = LMEIntermediateRep::from_lean(inv_neg_lean)?;
        let inv_neg_str = format!("{inv_neg_ir}");
        let manual_inv_neg = "(= (inv (- ?q)) (- (inv ?q)))".to_string();

        assert_eq!(inv_neg_str, manual_inv_neg);

        let mul_comm_lean =
            serde_json::from_str::<LeanExpr>(include_str!("../../test_assets/mul_comm.json"))
                .unwrap();
        let mul_comm_ir = LMEIntermediateRep::from_lean(mul_comm_lean)?;
        let mul_comm_str = format!("{mul_comm_ir}");
        let manual_mul_comm = "(= (* ?a ?b) (* ?b ?a))".to_string();

        assert_eq!(mul_comm_str, manual_mul_comm);

        let mul_inv_cancel_lean =
            serde_json::from_str::<LeanExpr>(include_str!("../../test_assets/mul_inv_cancel.json"))
                .unwrap();
        let mul_inv_cancel_ir = LMEIntermediateRep::from_lean(mul_inv_cancel_lean)?;
        let mul_inv_cancel_str = format!("{mul_inv_cancel_ir}");
        let manual_mul_inv_cancel = "(= (* ?a (inv ?a)) 1)".to_string();

        assert_eq!(mul_inv_cancel_str, manual_mul_inv_cancel);

        let neg_add_cancel_lean =
            serde_json::from_str::<LeanExpr>(include_str!("../../test_assets/neg_add_cancel.json"))
                .unwrap();
        let neg_add_cancel_ir = LMEIntermediateRep::from_lean(neg_add_cancel_lean)?;
        let neg_add_cancel_str = format!("{neg_add_cancel_ir}");
        let manual_neg_add_cancel = "(= (+ (- ?a) ?a) 0)".to_string();

        assert_eq!(neg_add_cancel_str, manual_neg_add_cancel);

        Ok(())
    }
}
