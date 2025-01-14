use egg::{define_language, Id};

use crate::JuniperBigRational;

define_language! {
    pub enum MathExpression {
        Constant(JuniperBigRational),
        "π" = Pi,
        Variable(char),
        ":=" = Assign([Id; 2]),
        "=" = Eq([Id; 2]),
        "+" = Add([Id; 2]),
        "-" = Sub([Id; 2]),
        "*" = Mul([Id; 2]),
        "/" = Div([Id; 2]),
        "^" = Pow([Id; 2]),
        "sqrt" = Sqrt(Id),
        "-" = Neg(Id),
        "inv" = Inv(Id),
        "sin" = Sin(Id),
        "cos" = Cos(Id),
        "anti-d" = Antiderivative([Id; 2]),
        "d" = Derivative([Id; 2]),
        "int" = Integral([Id; 4]),
    }
}
