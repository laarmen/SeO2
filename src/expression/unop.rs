use super::{LuaValue, Result, eval_expr, boolean_coercion};
use nom_lua53::op::UnOp;
use nom_lua53::Exp;

pub fn eval_unary_expr(operand: &Box<Exp>, operator: &UnOp) -> Result<LuaValue> {
    let operand = eval_expr(&operand)?;
    match *operator {
        UnOp::BoolNot => Ok(LuaValue::Boolean(!boolean_coercion(&operand))),
        _ => Ok(LuaValue::Nil)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use nom_lua53::Exp;
    use nom_lua53::string::StringLit;
    use nom_lua53::num::Numeral;
    use nom_lua53::op::UnOp;

    use std::borrow::Cow;

    #[test]
    fn test_negation() {
        // 1. + -1. == 0.
        let res = eval_unary_expr(&Box::new(Exp::Num(Numeral::Float(1.0))), &UnOp::BoolNot).unwrap();
        assert_eq!(res, LuaValue::Boolean(false));

        let res = eval_unary_expr(&Box::new(Exp::Num(Numeral::Int(0))), &UnOp::BoolNot).unwrap();
        assert_eq!(res, LuaValue::Boolean(false));

        let res = eval_unary_expr(&Box::new(Exp::Str(StringLit(Cow::from(&b""[..])))), &UnOp::BoolNot).unwrap();
        assert_eq!(res, LuaValue::Boolean(false));

        let res = eval_unary_expr(&Box::new(Exp::Nil), &UnOp::BoolNot).unwrap();
        assert_eq!(res, LuaValue::Boolean(true));
    }
}

