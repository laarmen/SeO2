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

