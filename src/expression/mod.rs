mod binop;
mod unop;

use super::{LuaError, Result};
use super::types::{LuaValue, LuaTable};

use nom_lua53;
use std;

// What do we say? We say "Merci Basile!"
fn lit_to_string(string: &nom_lua53::string::StringLit) -> String {
    return String::from_utf8_lossy(&(string.0)).to_string();
}

fn num_coercion(val: LuaValue) -> LuaValue {
    match val {
        LuaValue::Str(s) => {
            let parsed =  str::parse::<isize>(&s);
            match parsed {
                Ok(n) => LuaValue::Integer(n),
                Err(_) => {
                    let parsed =  str::parse::<f64>(&s);
                    match parsed {
                        Ok(n) => LuaValue::Float(n),
                        Err(_) => LuaValue::Str(s)
                    }
                }
            }
        },
        _ => val
    }
}

pub fn eval_expr(expr: &nom_lua53::Exp) -> Result<LuaValue> {
    match *expr {
        nom_lua53::Exp::Nil => Ok(LuaValue::Nil),
        nom_lua53::Exp::Bool(val) => Ok(LuaValue::Boolean(val)),
        nom_lua53::Exp::Num(val) => Ok(match val {
            nom_lua53::num::Numeral::Float(fl) => LuaValue::Float(fl),
            nom_lua53::num::Numeral::Int(i) => LuaValue::Integer(i),
        }),
        nom_lua53::Exp::BinExp(ref left, ref op, ref right) => binop::eval_binary_expr(left, right, &op),
        nom_lua53::Exp::UnExp(ref operator, ref operand) => unop::eval_unary_expr(operand, &operator),
        nom_lua53::Exp::Str(ref s) => Ok(LuaValue::Str(lit_to_string(s))),
        _ => Ok(LuaValue::Nil),
    }
}

pub fn boolean_coercion(val: &LuaValue) -> bool {
    match *val {
        LuaValue::Nil => false,
        LuaValue::Boolean(b) => b,
        _ => true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_coercion() {
        assert!(boolean_coercion(&LuaValue::Str("".to_owned())));
        assert!(boolean_coercion(&LuaValue::Integer(0)));
        assert!(boolean_coercion(&LuaValue::Integer(2)));
        assert!(!boolean_coercion(&LuaValue::Nil));
        assert!(!boolean_coercion(&LuaValue::Boolean(false)));
        assert!(boolean_coercion(&LuaValue::Boolean(true)));
    }
}

