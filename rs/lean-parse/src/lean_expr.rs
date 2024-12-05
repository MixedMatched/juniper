use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Name {
    Anonymous,
    Str { pre: Box<Name>, str: String },
    Num { pre: Box<Name>, i: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) struct LMVarId {
    name: Name,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) struct FVarId {
    name: Name,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) struct MVarId {
    name: Name,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Level {
    Zero,
    Succ(Box<Level>),
    Max(Box<Level>, Box<Level>),
    IMax(Box<Level>, Box<Level>),
    Param(Name),
    MVar(LMVarId),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum BinderInfo {
    Default,
    Implicit,
    StrictImplicit,
    InstImplicit,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "Expr")]
#[serde(rename_all = "lowercase")]
#[serde(rename_all_fields = "camelCase")]
pub(crate) enum LeanExpr {
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
        fun: Box<LeanExpr>, // needs to be named fn!!
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
}
