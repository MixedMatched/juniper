use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Literal {
    NatVal { val: u32 },
    StrVal { val: String },
}

type Name = String;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FVarId {
    name: Name,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MVarId {
    name: Name,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Zero,
    Succ(Box<Level>),
    Max(Box<Level>, Box<Level>),
    IMax(Box<Level>, Box<Level>),
    Param(Name),
    MVar(LMVarId),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BinderInfo {
    Default,
    Implicit,
    StrictImplicit,
    InstImplicit,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DataValue {
    OfString { v: String },
    OfBool { v: bool },
    OfName { v: Name },
    OfNat { v: u32 },
    OfInt { v: i32 },
    OfSyntax { v: Syntax },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MData(HashMap<Name, DataValue>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "Expr")]
#[serde(rename_all = "lowercase")]
#[serde(rename_all_fields = "camelCase")]
pub enum LeanExpr {
    BVar {
        de_bruijn_index: u32,
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
        us: Vec<Level>,
    },
    App {
        #[serde(rename = "fn")]
        function: Box<LeanExpr>, // needs to be named fn!!
        arg: Box<LeanExpr>,
    },
    Lam {
        binder_name: Name,
        binder_type: Box<LeanExpr>,
        body: Box<LeanExpr>,
        binder_info: BinderInfo,
    },
    #[serde(rename = "forallE")]
    ForallE {
        binder_name: Name,
        binder_type: Box<LeanExpr>,
        body: Box<LeanExpr>,
        binder_info: BinderInfo,
    },
    #[serde(rename = "letE")]
    LetE {
        decl_name: Name,
        #[serde(rename = "type")]
        typ: Box<LeanExpr>,
        value: Box<LeanExpr>,
        body: Box<LeanExpr>,
        non_dep: bool,
    },
    Lit(Literal),
    MData {
        data: MData,
        expr: Box<LeanExpr>,
    },
    Proj {
        type_name: Name,
        idx: u32,
        #[serde(rename = "struct")]
        structure: Box<LeanExpr>,
    },
}

#[cfg(test)]
mod tests {
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
    }
}
