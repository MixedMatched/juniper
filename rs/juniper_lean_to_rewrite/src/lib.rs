use std::fmt::Display;

use anyhow::{Error, Result};

use egg::Pattern;
use egg::Rewrite;
use juniper_math_expression::ConstantFold;
use juniper_math_expression::MathExpression;
use lean_parse::lean_expr::Literal;
use lean_parse::lean_expr::{LeanExpr, Name};
use num::BigInt;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct JuniperJsonEntry {
    name: Name,
    #[serde(rename = "type")]
    typ: LeanExpr,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
struct Hole;

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
    Const(LMEIntermediateConst),
    Var(Name),
    Forall {
        binder_name: Option<Name>,
        binder_type: Option<Name>,
        body: Option<Box<LMEIntermediateRep>>,
    },
    Eq {
        all_type: Option<Name>,
        in1: Option<Box<LMEIntermediateRep>>,
        in2: Option<Box<LMEIntermediateRep>>,
    },
    HBool {
        operator: Option<Name>,
        in1_type: Option<Name>,
        in2_type: Option<Name>,
        out_type: Option<Name>,
        inst: Option<Hole>,
        in1: Option<Box<LMEIntermediateRep>>,
        in2: Option<Box<LMEIntermediateRep>>,
    },
}

impl Display for LMEIntermediateRep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
        }
    }
}

impl LMEIntermediateRep {
    // convert a declName to an uninstantiated intermediate representation
    fn name_to_ir(name: Name) -> Result<LMEIntermediateRep> {
        match name.as_str() {
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
                binder_type: Some(Self::type_parse(*binder_type, de_bruijn_names.clone())?),
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

    // split apart intermediate representations at their top-level Eq (ignoring Foralls)
    fn split_at_top_eq(&self) -> Option<(LMEIntermediateRep, LMEIntermediateRep)> {
        match self {
            LMEIntermediateRep::Forall { body, .. } => {
                if let Some(body) = body {
                    body.split_at_top_eq()
                } else {
                    None
                }
            }
            LMEIntermediateRep::Eq { in1, in2, .. } => {
                if let Some(in1) = in1 {
                    if let Some(in2) = in2 {
                        Some(((**in1).clone(), (**in2).clone()))
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

    // convert intermediate representations into MathExpression Patterns
    fn to_math_expression(self) -> Result<Pattern<MathExpression>> {
        // this is really dumb, but I don't want to deal with RecExpr Ids
        Ok(format!("{self}").parse()?)
    }
}

// convert a list of named LeanExprs into egg MathExpression rewrite rules
pub fn lean_to_rewrites(
    lean_exprs: Vec<JuniperJsonEntry>,
) -> Result<Vec<Rewrite<MathExpression, ConstantFold>>> {
    let mut result = Vec::new();
    for JuniperJsonEntry { name, typ: expr } in lean_exprs {
        let intermediate = LMEIntermediateRep::from_lean(expr)?;
        if let Some((eq1, eq2)) = intermediate.split_at_top_eq() {
            result.push(
                Rewrite::new(name, eq1.to_math_expression()?, eq2.to_math_expression()?)
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

    #[test]
    fn simple() {
        let lean_statement = serde_json::from_str::<LeanExpr>(include_str!("test.json")).unwrap();

        let ir = LMEIntermediateRep::from_lean(lean_statement).unwrap();

        println!("{ir}");
    }
}
