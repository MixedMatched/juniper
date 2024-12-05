enum Name {
    Anonymous,
    Str { pre: Box<Name>, str: String },
    Num { pre: Box<Name>, i: u32 },
}

struct LMVarId {
    name: Name,
}

struct FVarId {
    name: Name,
}

struct MVarId {
    name: Name,
}

enum Level {
    Zero,
    Succ(Box<Level>),
    Max(Box<Level>, Box<Level>),
    IMax(Box<Level>, Box<Level>),
    Param(Name),
    MVar(LMVarId),
}

enum BinderInfo {
    Default,
    Implicit,
    StrictImplicit,
    InstImplicit,
}

enum LeanExpr {
    BVar {
        deBruijnIndex: u32,
    },
    FVar {
        fvarId: FVarId,
    },
    MVar {
        mvarId: MVarId,
    },
    Sort {
        u: Level,
    },
    Const {
        declName: Name,
        us: Vec<Level>,
    },
    App {
        fun: Box<LeanExpr>, // needs to be named fn!!
        arg: Box<LeanExpr>,
    },
    Lam {
        binderName: Name,
        binderType: Box<LeanExpr>,
        body: Box<LeanExpr>,
        binderInfo: BinderInfo,
    },
    ForallE {
        binderName: Name,
        binderType: Box<LeanExpr>,
        body: Box<LeanExpr>,
        binderInfo: BinderInfo,
    },
}

