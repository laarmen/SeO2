use super::{boolean_coercion, eval_expr, num_coercion, LuaState, LuaValue, Result};
use nom_lua53::op::UnOp;
use nom_lua53::Exp;

use LuaError::*;

pub fn eval_unary_expr(operand: &Box<Exp>, operator: &UnOp, ctx: &LuaState) -> Result<LuaValue> {
    let operand = eval_expr(&operand, ctx)?;
    match *operator {
        UnOp::BoolNot => Ok(LuaValue::Boolean(!boolean_coercion(&operand))),
        UnOp::Minus => match num_coercion(operand) {
            LuaValue::Integer(i) => Ok(LuaValue::Integer(-i)),
            LuaValue::Float(f) => Ok(LuaValue::Float(-f)),
            _ => Err(TypeError(
                "Trying to do arithmetic on a non-numerical value.".to_owned(),
            )),
        },
        UnOp::Length => match operand {
            LuaValue::Str(s) => Ok(LuaValue::Integer(s.len() as isize)),
            LuaValue::Table(t) => Ok(LuaValue::Integer(t.sequence_border() as isize)),
            _ => Err(TypeError(
                "Trying to do get size on an unsupported type.".to_owned(),
            )),
        },
        UnOp::BitNot => match num_coercion(operand) {
            LuaValue::Integer(i) => Ok(LuaValue::Integer(!i)),
            LuaValue::Float(f) => Ok(LuaValue::Integer(!(f as isize))),
            _ => Err(TypeError(
                "Trying to do bitwise inversion on a non-numerical value.".to_owned(),
            )),
        },
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
    fn test_math_negation() {
        let ctx = LuaState::new();
        // 1. + -1. == 0.
        let res =
            eval_unary_expr(&Box::new(Exp::Num(Numeral::Float(1.0))), &UnOp::Minus, &ctx).unwrap();
        assert_eq!(res, LuaValue::Float(-1.));

        let res =
            eval_unary_expr(&Box::new(Exp::Num(Numeral::Int(0))), &UnOp::Minus, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(0));

        let res =
            eval_unary_expr(&Box::new(Exp::Num(Numeral::Int(-4))), &UnOp::Minus, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(4));

        let res = eval_unary_expr(
            &Box::new(Exp::Str(StringLit(Cow::from(&b"5.5"[..])))),
            &UnOp::Minus,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Float(-5.5));

        let res = eval_unary_expr(&Box::new(Exp::Bool(true)), &UnOp::Minus, &ctx).unwrap_err();
        assert!(match res {
            TypeError(_) => true,
            _ => false,
        });
    }

    #[test]
    fn test_bool_negation() {
        let ctx = LuaState::new();
        // 1. + -1. == 0.
        let res = eval_unary_expr(
            &Box::new(Exp::Num(Numeral::Float(1.0))),
            &UnOp::BoolNot,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Boolean(false));

        let res =
            eval_unary_expr(&Box::new(Exp::Num(Numeral::Int(0))), &UnOp::BoolNot, &ctx).unwrap();
        assert_eq!(res, LuaValue::Boolean(false));

        let res = eval_unary_expr(
            &Box::new(Exp::Str(StringLit(Cow::from(&b""[..])))),
            &UnOp::BoolNot,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Boolean(false));

        let res = eval_unary_expr(&Box::new(Exp::Nil), &UnOp::BoolNot, &ctx).unwrap();
        assert_eq!(res, LuaValue::Boolean(true));
    }

    #[test]
    fn test_bool_length() {
        let ctx = LuaState::new();
        let res = eval_unary_expr(
            &Box::new(Exp::Str(StringLit(Cow::from(&b"1234"[..])))),
            &UnOp::Length,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Integer(4));

        let res = eval_unary_expr(&Box::new(Exp::Bool(true)), &UnOp::Length, &ctx).unwrap_err();
        assert!(match res {
            TypeError(_) => true,
            _ => false,
        });
    }

    #[test]
    fn test_bitwise_inversion() {
        let ctx = LuaState::new();
        let res =
            eval_unary_expr(&Box::new(Exp::Num(Numeral::Int(0))), &UnOp::BitNot, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(-1));

        let res = eval_unary_expr(&Box::new(Exp::Bool(true)), &UnOp::BitNot, &ctx).unwrap_err();
        assert!(match res {
            TypeError(_) => true,
            _ => false,
        });
    }
}
