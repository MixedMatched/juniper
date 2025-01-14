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
            LMEIntermediateDefinedConst::Pi => write!(f, "π"),
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
            LMEIntermediateConst::OfNat { val, .. } => {
                if let Some(val) = val {
                    write!(f, "{val}")
                } else {
                    write!(f, "")
                }
            }
            LMEIntermediateConst::OfScientific {
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
        binder_type: Option<Box<LMEIntermediateRep>>,
        body: Option<Box<LMEIntermediateRep>>,
    },
    Eq {
        all_type: Option<Name>,
        in1: Option<Box<LMEIntermediateRep>>,
        in2: Option<Box<LMEIntermediateRep>>,
    },
    Ne {
        all_type: Option<Name>,
        in1: Option<Box<LMEIntermediateRep>>,
        in2: Option<Box<LMEIntermediateRep>>,
    },
    HBool {
        operator: Option<String>,
        in1_type: Option<Name>,
        in2_type: Option<Name>,
        out_type: Option<Name>,
        inst: Option<Hole>,
        in1: Option<Box<LMEIntermediateRep>>,
        in2: Option<Box<LMEIntermediateRep>>,
    },
    TUnary {
        operator: Option<String>,
        all_type: Option<Name>,
        inst: Option<Hole>,
        in1: Option<Box<LMEIntermediateRep>>,
    },
    IUnary {
        operator: Option<String>,
        in1: Option<Box<LMEIntermediateRep>>,
    },
}

impl Display for LMEIntermediateRep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LMEIntermediateRep::DefinedConst(dc) => write!(f, "{dc}"),
            LMEIntermediateRep::Const(c) => write!(f, "{c}"),
            LMEIntermediateRep::Var(name) => write!(f, "?{name}"),
            LMEIntermediateRep::Forall { body, .. } => {
                if let Some(body) = body {
                    write!(f, "{body}")
                } else {
                    write!(f, "")
                }
            }
            LMEIntermediateRep::Eq { in1, in2, .. } => {
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
            LMEIntermediateRep::Ne { in1, in2, .. } => {
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
            LMEIntermediateRep::HBool {
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
            LMEIntermediateRep::TUnary { operator, in1, .. }
            | LMEIntermediateRep::IUnary { operator, in1 } => {
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
    fn name_to_ir(name: Name) -> Result<LMEIntermediateRep> {
        match name.as_str() {
            "Real.pi" => Ok(LMEIntermediateRep::DefinedConst(
                LMEIntermediateDefinedConst::Pi,
            )),
            "OfScientific.ofScientific" => Ok(LMEIntermediateRep::Const(
                LMEIntermediateConst::OfScientific {
                    out_type: None,
                    inst: None,
                    mantissa: None,
                    exponent_sign: None,
                    decimal_exponent: None,
                },
            )),
            "OfNat.ofNat" => Ok(LMEIntermediateRep::Const(LMEIntermediateConst::OfNat {
                out_type: None,
                val: None,
                inst: None,
            })),
            "Eq" => Ok(LMEIntermediateRep::Eq {
                all_type: None,
                in1: None,
                in2: None,
            }),
            "Ne" => Ok(LMEIntermediateRep::Ne {
                all_type: None,
                in1: None,
                in2: None,
            }),
            "HAdd.hAdd" => Ok(LMEIntermediateRep::HBool {
                operator: Some("+".to_string()),
                in1_type: None,
                in2_type: None,
                out_type: None,
                inst: None,
                in1: None,
                in2: None,
            }),
            "HSub.hSub" => Ok(LMEIntermediateRep::HBool {
                operator: Some("-".to_string()),
                in1_type: None,
                in2_type: None,
                out_type: None,
                inst: None,
                in1: None,
                in2: None,
            }),
            "HMul.hMul" => Ok(LMEIntermediateRep::HBool {
                operator: Some("*".to_string()),
                in1_type: None,
                in2_type: None,
                out_type: None,
                inst: None,
                in1: None,
                in2: None,
            }),
            "HDiv.hDiv" => Ok(LMEIntermediateRep::HBool {
                operator: Some("/".to_string()),
                in1_type: None,
                in2_type: None,
                out_type: None,
                inst: None,
                in1: None,
                in2: None,
            }),
            "HPow.hPow" => Ok(LMEIntermediateRep::HBool {
                operator: Some("^".to_string()),
                in1_type: None,
                in2_type: None,
                out_type: None,
                inst: None,
                in1: None,
                in2: None,
            }),
            "Neg.neg" => Ok(LMEIntermediateRep::TUnary {
                operator: Some("-".to_string()),
                all_type: None,
                inst: None,
                in1: None,
            }),
            "Inv.inv" => Ok(LMEIntermediateRep::TUnary {
                operator: Some("inv".to_string()),
                all_type: None,
                inst: None,
                in1: None,
            }),
            "Real.sin" => Ok(LMEIntermediateRep::IUnary {
                operator: Some("sin".to_string()),
                in1: None,
            }),
            "Real.cos" => Ok(LMEIntermediateRep::IUnary {
                operator: Some("cos".to_string()),
                in1: None,
            }),
            "Real.sqrt" => Ok(LMEIntermediateRep::IUnary {
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
    fn app_state_next(
        current: LMEIntermediateRep,
        arg: LeanExpr,
        de_bruijn_names: Vec<Name>,
    ) -> Result<LMEIntermediateRep> {
        // because the arguments are ordered, we only have to specify that one argument is None
        // also this is really ugly, but that's mostly bc rust enum structs lack default support lol
        Ok(match current {
            LMEIntermediateRep::Const(LMEIntermediateConst::OfNat { out_type: None, .. }) => {
                LMEIntermediateRep::Const(LMEIntermediateConst::OfNat {
                    out_type: Some(Self::type_parse(arg, de_bruijn_names.clone())?),
                    val: None,
                    inst: None,
                })
            }
            LMEIntermediateRep::Const(LMEIntermediateConst::OfNat {
                out_type,
                val: None,
                ..
            }) => LMEIntermediateRep::Const(LMEIntermediateConst::OfNat {
                out_type,
                val: match arg {
                    LeanExpr::Lit(Literal::NatVal { val }) => Some(val.into()),
                    _ => return Err(Error::msg(format!("bad OfNat val: {}", arg))),
                },
                inst: None,
            }),
            LMEIntermediateRep::Const(LMEIntermediateConst::OfNat {
                out_type,
                val,
                inst: None,
            }) => LMEIntermediateRep::Const(LMEIntermediateConst::OfNat {
                out_type,
                val,
                inst: Some(Hole),
            }),
            LMEIntermediateRep::Const(LMEIntermediateConst::OfScientific {
                out_type: None,
                ..
            }) => LMEIntermediateRep::Const(LMEIntermediateConst::OfScientific {
                out_type: Some(Self::type_parse(arg, de_bruijn_names.clone())?),
                inst: None,
                mantissa: None,
                exponent_sign: None,
                decimal_exponent: None,
            }),
            LMEIntermediateRep::Const(LMEIntermediateConst::OfScientific {
                out_type,
                inst: None,
                ..
            }) => LMEIntermediateRep::Const(LMEIntermediateConst::OfScientific {
                out_type,
                inst: Some(Hole),
                mantissa: None,
                exponent_sign: None,
                decimal_exponent: None,
            }),
            LMEIntermediateRep::Const(LMEIntermediateConst::OfScientific {
                out_type,
                inst,
                mantissa: None,
                ..
            }) => LMEIntermediateRep::Const(LMEIntermediateConst::OfScientific {
                out_type,
                inst,
                mantissa: match arg {
                    LeanExpr::Lit(Literal::NatVal { val }) => Some(val.into()),
                    _ => return Err(Error::msg(format!("bad mantissa: {}", arg))),
                },
                exponent_sign: None,
                decimal_exponent: None,
            }),
            LMEIntermediateRep::Const(LMEIntermediateConst::OfScientific {
                out_type,
                inst,
                mantissa,
                exponent_sign: None,
                ..
            }) => LMEIntermediateRep::Const(LMEIntermediateConst::OfScientific {
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
            LMEIntermediateRep::Const(LMEIntermediateConst::OfScientific {
                out_type,
                inst,
                mantissa,
                exponent_sign,
                decimal_exponent: None,
            }) => LMEIntermediateRep::Const(LMEIntermediateConst::OfScientific {
                out_type,
                inst,
                mantissa,
                exponent_sign,
                decimal_exponent: match arg {
                    LeanExpr::Lit(Literal::NatVal { val }) => Some(val.try_into()?),
                    _ => return Err(Error::msg(format!("bad decimal_exponent: {}", arg))),
                },
            }),
            LMEIntermediateRep::Eq { all_type: None, .. } => LMEIntermediateRep::Eq {
                all_type: Some(Self::type_parse(arg, de_bruijn_names.clone())?),
                in1: None,
                in2: None,
            },
            LMEIntermediateRep::Eq {
                all_type,
                in1: None,
                ..
            } => LMEIntermediateRep::Eq {
                all_type,
                in1: Some(Box::new(Self::from_lean_recursive(
                    arg,
                    de_bruijn_names.clone(),
                )?)),
                in2: None,
            },
            LMEIntermediateRep::Eq {
                all_type,
                in1,
                in2: None,
            } => LMEIntermediateRep::Eq {
                all_type,
                in1,
                in2: Some(Box::new(Self::from_lean_recursive(
                    arg,
                    de_bruijn_names.clone(),
                )?)),
            },
            LMEIntermediateRep::Ne { all_type: None, .. } => LMEIntermediateRep::Ne {
                all_type: Some(Self::type_parse(arg, de_bruijn_names.clone())?),
                in1: None,
                in2: None,
            },
            LMEIntermediateRep::Ne {
                all_type,
                in1: None,
                ..
            } => LMEIntermediateRep::Ne {
                all_type,
                in1: Some(Box::new(Self::from_lean_recursive(
                    arg,
                    de_bruijn_names.clone(),
                )?)),
                in2: None,
            },
            LMEIntermediateRep::Ne {
                all_type,
                in1,
                in2: None,
            } => LMEIntermediateRep::Ne {
                all_type,
                in1,
                in2: Some(Box::new(Self::from_lean_recursive(
                    arg,
                    de_bruijn_names.clone(),
                )?)),
            },
            LMEIntermediateRep::HBool {
                operator,
                in1_type: None,
                ..
            } => LMEIntermediateRep::HBool {
                operator,
                in1_type: Some(Self::type_parse(arg, de_bruijn_names.clone())?),
                in2_type: None,
                out_type: None,
                inst: None,
                in1: None,
                in2: None,
            },
            LMEIntermediateRep::HBool {
                operator,
                in1_type,
                in2_type: None,
                ..
            } => LMEIntermediateRep::HBool {
                operator,
                in1_type,
                in2_type: Some(Self::type_parse(arg, de_bruijn_names.clone())?),
                out_type: None,
                inst: None,
                in1: None,
                in2: None,
            },
            LMEIntermediateRep::HBool {
                operator,
                in1_type,
                in2_type,
                out_type: None,
                ..
            } => LMEIntermediateRep::HBool {
                operator,
                in1_type,
                in2_type,
                out_type: Some(Self::type_parse(arg, de_bruijn_names.clone())?),
                inst: None,
                in1: None,
                in2: None,
            },
            LMEIntermediateRep::HBool {
                operator,
                in1_type,
                in2_type,
                out_type,
                inst: None,
                ..
            } => LMEIntermediateRep::HBool {
                operator,
                in1_type,
                in2_type,
                out_type,
                inst: Some(Hole),
                in1: None,
                in2: None,
            },
            LMEIntermediateRep::HBool {
                operator,
                in1_type,
                in2_type,
                out_type,
                inst,
                in1: None,
                ..
            } => LMEIntermediateRep::HBool {
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
            LMEIntermediateRep::HBool {
                operator,
                in1_type,
                in2_type,
                out_type,
                inst,
                in1,
                in2: None,
            } => LMEIntermediateRep::HBool {
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
            LMEIntermediateRep::TUnary {
                operator,
                all_type: None,
                ..
            } => LMEIntermediateRep::TUnary {
                operator,
                all_type: Some(Self::type_parse(arg, de_bruijn_names.clone())?),
                inst: None,
                in1: None,
            },
            LMEIntermediateRep::TUnary {
                operator,
                all_type,
                inst: None,
                ..
            } => LMEIntermediateRep::TUnary {
                operator,
                all_type,
                inst: Some(Hole),
                in1: None,
            },
            LMEIntermediateRep::TUnary {
                operator,
                all_type,
                inst,
                in1: None,
            } => LMEIntermediateRep::TUnary {
                operator,
                all_type,
                inst,
                in1: Some(Box::new(Self::from_lean_recursive(
                    arg,
                    de_bruijn_names.clone(),
                )?)),
            },
            LMEIntermediateRep::IUnary {
                operator,
                in1: None,
            } => LMEIntermediateRep::IUnary {
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
    ) -> Result<LMEIntermediateRep> {
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
    fn from_lean_recursive(
        expr: LeanExpr,
        de_bruijn_names: Vec<Name>,
    ) -> Result<LMEIntermediateRep> {
        match expr {
            LeanExpr::ForallE {
                binder_name,
                binder_type,
                body,
                binder_info: _,
            } => Ok(LMEIntermediateRep::Forall {
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
            LeanExpr::BVar { de_bruijn_index } => Ok(LMEIntermediateRep::Var(
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
                    Ok(LMEIntermediateRep::DefinedConst(
                        LMEIntermediateDefinedConst::Pi,
                    ))
                }
            }
            _ => Err(Error::msg(format!(
                "improper top-level structure: {}",
                expr
            ))),
        }
    }

    // convert LeanExpr into an intermediate representation
    fn from_lean(expr: LeanExpr) -> Result<LMEIntermediateRep> {
        Self::from_lean_recursive(expr, Vec::new())
    }

    // split apart intermediate representations at their top-level Eq (collecting condition Foralls and ignoring others)
    fn split_at_top_eq(
        &self,
        conditions: Vec<LMEIntermediateRep>,
    ) -> Option<(
        Vec<LMEIntermediateRep>,
        LMEIntermediateRep,
        LMEIntermediateRep,
    )> {
        match self {
            LMEIntermediateRep::Forall {
                body, binder_type, ..
            } => {
                if let Some(body) = body {
                    if let Some(binder_type) = binder_type {
                        match *binder_type.clone() {
                            LMEIntermediateRep::Const(_) | LMEIntermediateRep::DefinedConst(_) => {
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
            LMEIntermediateRep::Eq { in1, in2, .. } => {
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
            LMEIntermediateRep::Eq { in1: Some(in1), in2: Some(in2), .. } => {
                let in1 = in1.to_math_expression()?;
                let in2 = in2.to_math_expression()?;

                let check_eq = ConditionEqual::new(in1, in2);

                Ok(Box::new(move |egraph, id, subst| {
                    check_eq.check(egraph, id, subst)
                }))
            },
            LMEIntermediateRep::Ne {  in1: Some(in1), in2: Some(in2), .. } => {
                let in1 = in1.to_math_expression()?;
                let in2 = in2.to_math_expression()?;

                let check_eq = ConditionEqual::new(in1, in2);

                // this is technically unsound (if, in the future, in1 = in2, just not when you're
                // currently checking)
                Ok(Box::new(move |egraph, id, subst| {
                    !check_eq.check(egraph, id, subst)
                }))
            }
            _ => Err(Error::msg(format!("LMEIntermediateRep::as_condition could not successfully convert the condition {self} into a closure"))),
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

    #[test]
    fn simple() {
        let lean_statement = serde_json::from_str::<LeanExpr>(include_str!("test.json")).unwrap();

        let ir = LMEIntermediateRep::from_lean(lean_statement).unwrap();

        println!("{ir}");
    }
}
