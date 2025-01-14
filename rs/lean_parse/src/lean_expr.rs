use std::collections::HashMap;
use std::fmt::Display;

use display_tree::{write_tree, DisplayTree};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Literal {
    NatVal { val: u64 },
    StrVal { val: String },
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::NatVal { val } => write!(f, "Literal {{{val}}}"),
            Literal::StrVal { val } => write!(f, "Literal {{{val}}}"),
        }
    }
}

pub type Name = String;
// there seems to already be an ToJson instance for Name that makes it a string?
/*
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Name {
    Anonymous,
    Str { pre: Box<Name>, str: String },
    Num { pre: Box<Name>, i: u32 },
}
*/

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LMVarId {
    name: Name,
}

impl Display for LMVarId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LMVarId {{{}}}", self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FVarId {
    name: Name,
}

impl Display for FVarId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FVarId {{{}}}", self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MVarId {
    name: Name,
}

impl Display for MVarId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MVarId {{{}}}", self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, DisplayTree)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Zero,
    Succ(#[tree] Box<Level>),
    Max(#[tree] Box<Level>, #[tree] Box<Level>),
    IMax(#[tree] Box<Level>, #[tree] Box<Level>),
    Param(Name),
    MVar(LMVarId),
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_tree!(f, *self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BinderInfo {
    Default,
    Implicit,
    StrictImplicit,
    InstImplicit,
}

impl Display for BinderInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BinderInfo::{}",
            match self {
                BinderInfo::Default => "Default",
                BinderInfo::Implicit => "Implicit",
                BinderInfo::StrictImplicit => "StrictImplicit",
                BinderInfo::InstImplicit => "InstImplicit",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
// hell no.
pub enum Syntax {
    Missing,
    Node {},
    Atom {},
    Ident {},
}

impl Display for Syntax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Syntax::{}",
            match self {
                Syntax::Missing => "Missing",
                Syntax::Ident {} => "Ident",
                Syntax::Atom {} => "Atom",
                Syntax::Node {} => "Node",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DataValue {
    OfString { v: String },
    OfBool { v: bool },
    OfName { v: Name },
    OfNat { v: u64 },
    OfInt { v: i64 },
    OfSyntax { v: Syntax },
}

impl Display for DataValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MData(HashMap<Name, DataValue>);

impl Display for MData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, DisplayTree)]
#[serde(rename = "Expr")]
#[serde(rename_all = "lowercase")]
#[serde(rename_all_fields = "camelCase")]
pub enum LeanExpr {
    BVar {
        de_bruijn_index: u64,
    },
    FVar {
        fvar_id: FVarId,
    },
    MVar {
        mvar_id: MVarId,
    },
    Sort {
        u: Level,
    },
    Const {
        decl_name: Name,
        #[ignore_field]
        us: Vec<Level>,
    },
    App {
        #[serde(rename = "fn")]
        #[tree]
        function: Box<LeanExpr>, // needs to be named fn!!
        #[tree]
        arg: Box<LeanExpr>,
    },
    Lam {
        binder_name: Name,
        #[tree]
        binder_type: Box<LeanExpr>,
        #[tree]
        body: Box<LeanExpr>,
        binder_info: BinderInfo,
    },
    #[serde(rename = "forallE")]
    ForallE {
        binder_name: Name,
        #[tree]
        binder_type: Box<LeanExpr>,
        #[tree]
        body: Box<LeanExpr>,
        binder_info: BinderInfo,
    },
    #[serde(rename = "letE")]
    LetE {
        decl_name: Name,
        #[serde(rename = "type")]
        #[tree]
        typ: Box<LeanExpr>,
        #[tree]
        value: Box<LeanExpr>,
        #[tree]
        body: Box<LeanExpr>,
        non_dep: bool,
    },
    Lit(Literal),
    MData {
        data: MData,
        #[tree]
        expr: Box<LeanExpr>,
    },
    Proj {
        type_name: Name,
        idx: u64,
        #[serde(rename = "struct")]
        #[tree]
        structure: Box<LeanExpr>,
    },
}

impl Display for LeanExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_tree!(f, *self)
    }
}

#[cfg(test)]
mod tests {
    use display_tree::println_tree;

    use super::LeanExpr;

    #[test]
    fn x() {
        let json = r#"{"forallE":
 {"body":
  {"forallE":
   {"body":
    {"app":
     {"fn":
      {"app":
       {"fn":
        {"app":
         {"fn": {"const": {"us": [{"succ": "zero"}], "declName": "Eq"}},
          "arg": {"const": {"us": [], "declName": "Rat"}}}},
        "arg":
        {"app":
         {"fn":
          {"app":
           {"fn":
            {"app":
             {"fn":
              {"app":
               {"fn":
                {"app":
                 {"fn":
                  {"app":
                   {"fn": {"const": {"us": ["zero", "zero", "zero"], "declName": "HAdd.hAdd"}},
                    "arg": {"const": {"us": [], "declName": "Rat"}}}},
                  "arg": {"const": {"us": [], "declName": "Rat"}}}},
                "arg": {"const": {"us": [], "declName": "Rat"}}}},
              "arg":
              {"app":
               {"fn":
                {"app":
                 {"fn": {"const": {"us": ["zero"], "declName": "instHAdd"}},
                  "arg": {"const": {"us": [], "declName": "Rat"}}}},
                "arg": {"const": {"us": [], "declName": "Rat.instAdd"}}}}}},
            "arg": {"bvar": {"deBruijnIndex": 1}}}},
          "arg": {"bvar": {"deBruijnIndex": 0}}}}}},
      "arg":
      {"app":
       {"fn":
        {"app":
         {"fn":
          {"app":
           {"fn":
            {"app":
             {"fn":
              {"app":
               {"fn":
                {"app":
                 {"fn": {"const": {"us": ["zero", "zero", "zero"], "declName": "HAdd.hAdd"}},
                  "arg": {"const": {"us": [], "declName": "Rat"}}}},
                "arg": {"const": {"us": [], "declName": "Rat"}}}},
              "arg": {"const": {"us": [], "declName": "Rat"}}}},
            "arg":
            {"app":
             {"fn":
              {"app":
               {"fn": {"const": {"us": ["zero"], "declName": "instHAdd"}},
                "arg": {"const": {"us": [], "declName": "Rat"}}}},
              "arg": {"const": {"us": [], "declName": "Rat.instAdd"}}}}}},
          "arg": {"bvar": {"deBruijnIndex": 0}}}},
        "arg": {"bvar": {"deBruijnIndex": 1}}}}}},
    "binderType": {"const": {"us": [], "declName": "Rat"}},
    "binderName": "b",
    "binderInfo": "default"}},
  "binderType": {"const": {"us": [], "declName": "Rat"}},
  "binderName": "a",
  "binderInfo": "default"}}"#;
        let obj: LeanExpr = serde_json::from_str(&json).unwrap();
        println!("{:?}", obj);
        println!("{}", serde_json::to_string(&obj).unwrap());

        println_tree!(obj);
    }
}
