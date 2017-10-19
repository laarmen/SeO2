mod binop;

use super::{LuaError, LuaValue, Result};

use nom_lua53;
use std;

// What do we say? We say "Merci Basile!"
fn lit_to_string(string: &nom_lua53::string::StringLit) -> String {
    return String::from_utf8_lossy(&(string.0)).to_string();
}

fn string_to_num_coercion(val: LuaValue) -> LuaValue {
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
        nom_lua53::Exp::Str(ref s) => Ok(LuaValue::Str(lit_to_string(s))),
        _ => Ok(LuaValue::Nil),
    }
}
