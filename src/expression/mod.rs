mod binop;
mod unop;
pub mod prefixexp;

use super::{var_to_string, LuaError, Result};
use super::types::{LuaState, LuaTable, LuaValue, Number};

use nom_lua53;
use std;

// What do we say? We say "Merci Basile!"
fn lit_to_string(string: &nom_lua53::string::StringLit) -> String {
    return String::from_utf8_lossy(&(string.0)).to_string();
}

pub fn num_coercion(val: LuaValue) -> LuaValue {
    match val {
        LuaValue::Str(s) => {
            let parsed = str::parse::<isize>(&s);
            match parsed {
                Ok(n) => LuaValue::Number(Number::Int(n)),
                Err(_) => {
                    let parsed = str::parse::<f64>(&s);
                    match parsed {
                        Ok(n) => LuaValue::Number(Number::Float(n)),
                        Err(_) => LuaValue::Str(s),
                    }
                }
            }
        }
        _ => val,
    }
}

pub fn eval_expr(expr: &nom_lua53::Exp, ctx: &LuaState) -> Result<LuaValue> {
    match *expr {
        nom_lua53::Exp::Nil => Ok(LuaValue::Nil),
        nom_lua53::Exp::Bool(val) => Ok(LuaValue::Boolean(val)),
        nom_lua53::Exp::Num(val) => Ok(match val {
            nom_lua53::num::Numeral::Float(fl) => LuaValue::Number(Number::Float(fl)),
            nom_lua53::num::Numeral::Int(i) => LuaValue::Number(Number::Int(i)),
        }),
        nom_lua53::Exp::BinExp(ref left, ref op, ref right) => {
            binop::eval_binary_expr(left, right, &op, ctx)
        }
        nom_lua53::Exp::UnExp(ref operator, ref operand) => {
            unop::eval_unary_expr(operand, &operator, ctx)
        }
        nom_lua53::Exp::Str(ref s) => Ok(LuaValue::Str(lit_to_string(s))),
        nom_lua53::Exp::Table(ref t) => eval_inline_table(t, ctx),
        nom_lua53::Exp::PrefixExp(ref e) => {
            prefixexp::eval_prefix_expr(&e.prefix, &e.suffix_chain, ctx)
        }
        nom_lua53::Exp::Ellipses => Err(LuaError::NotImplementedError),
        nom_lua53::Exp::FuncCall(_) => Err(LuaError::NotImplementedError),
        nom_lua53::Exp::Lambda(_) => Err(LuaError::NotImplementedError),
    }
}

fn eval_inline_table(src: &nom_lua53::TableLit, ctx: &LuaState) -> Result<LuaValue> {
    // First, we count how many positional epxressions there are to only do one allocation.
    let mut sequence_count = 0;
    for field in src.into_iter() {
        match field {
            &nom_lua53::Field::PosAssign(_) => sequence_count = sequence_count + 1,
            _ => (),
        }
    }

    let ret = LuaTable::with_capacity(ctx.get_ref_id(), sequence_count);
    let mut next_index = 1;
    for field in src.iter() {
        match *field {
            nom_lua53::Field::PosAssign(ref exp) => {
                ret.set(&LuaValue::Number(Number::Int(next_index)), &eval_expr(exp, ctx)?)?;
                next_index = next_index + 1;
            }
            nom_lua53::Field::ExpAssign(ref key, ref value) => {
                let key = eval_expr(key, ctx)?;
                let value = eval_expr(value, ctx)?;
                ret.set(&key, &value)?;
            }
            nom_lua53::Field::NameAssign(ref key, ref value) => {
                let key = var_to_string(key);
                let value = eval_expr(value, ctx)?;
                ret.set(&LuaValue::Str(key), &value)?;
            }
        }
    }
    return Ok(LuaValue::Table(ret));
}

pub fn boolean_coercion(val: &LuaValue) -> bool {
    match *val {
        LuaValue::Nil => false,
        LuaValue::Boolean(b) => b,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_coercion() {
        assert!(boolean_coercion(&LuaValue::Str("".to_owned())));
        assert!(boolean_coercion(&LuaValue::Number(Number::Int(0))));
        assert!(boolean_coercion(&LuaValue::Number(Number::Int(2))));
        assert!(!boolean_coercion(&LuaValue::Nil));
        assert!(!boolean_coercion(&LuaValue::Boolean(false)));
        assert!(boolean_coercion(&LuaValue::Boolean(true)));
    }
}
