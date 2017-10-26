use super::{LuaValue, Result, eval_expr, boolean_coercion, num_coercion};
use nom_lua53::op::UnOp;
use nom_lua53::Exp;

use LuaError::*;

pub fn eval_unary_expr(operand: &Box<Exp>, operator: &UnOp) -> Result<LuaValue> {
    let operand = eval_expr(&operand)?;
    match *operator {
        UnOp::BoolNot => Ok(LuaValue::Boolean(!boolean_coercion(&operand))),
        UnOp::Minus => {
            match num_coercion(operand) {
                LuaValue::Integer(i) => Ok(LuaValue::Integer(-i)),
                LuaValue::Float(f) => Ok(LuaValue::Float(-f)),
                _ => Err(TypeError("Trying to do arithmetic on a non-numerical value.".to_owned()))
            }
        },
        UnOp::Length => {
            match operand {
                LuaValue::Str(s) => Ok(LuaValue::Integer(s.len() as isize)),
                _ => Err(TypeError("Trying to do get size on an unsupported type.".to_owned()))
            }
        },
        UnOp::BitNot => {
            match num_coercion(operand) {
                LuaValue::Integer(i) => Ok(LuaValue::Integer(!i)),
                LuaValue::Float(f) => Ok(LuaValue::Integer(!(f as isize))),
                _ => Err(TypeError("Trying to do bitwise inversion on a non-numerical value.".to_owned()))
            }
        }
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
        // 1. + -1. == 0.
        let res = eval_unary_expr(&Box::new(Exp::Num(Numeral::Float(1.0))), &UnOp::Minus).unwrap();
        assert_eq!(res, LuaValue::Float(-1.));

        let res = eval_unary_expr(&Box::new(Exp::Num(Numeral::Int(0))), &UnOp::Minus).unwrap();
        assert_eq!(res, LuaValue::Integer(0));

        let res = eval_unary_expr(&Box::new(Exp::Num(Numeral::Int(-4))), &UnOp::Minus).unwrap();
        assert_eq!(res, LuaValue::Integer(4));

        let res = eval_unary_expr(&Box::new(Exp::Str(StringLit(Cow::from(&b"5.5"[..])))), &UnOp::Minus).unwrap();
        assert_eq!(res, LuaValue::Float(-5.5));

        let res = eval_unary_expr(&Box::new(Exp::Bool(true)), &UnOp::Minus).unwrap_err();
        assert!(match res { TypeError(_) => true, _ => false });
    }

    #[test]
    fn test_bool_negation() {
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

    #[test]
    fn test_bool_length() {
        let res = eval_unary_expr(&Box::new(Exp::Str(StringLit(Cow::from(&b"1234"[..])))), &UnOp::Length).unwrap();
        assert_eq!(res, LuaValue::Integer(4));

        let res = eval_unary_expr(&Box::new(Exp::Bool(true)), &UnOp::Length).unwrap_err();
        assert!(match res { TypeError(_) => true, _ => false });
    }

    #[test]
    fn test_bitwise_inversion() {
        let res = eval_unary_expr(&Box::new(Exp::Num(Numeral::Int(0))), &UnOp::BitNot).unwrap();
        assert_eq!(res, LuaValue::Integer(-1));

        let res = eval_unary_expr(&Box::new(Exp::Bool(true)), &UnOp::BitNot).unwrap_err();
        assert!(match res { TypeError(_) => true, _ => false });
    }
}

