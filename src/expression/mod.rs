mod operator;
use super::*;

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

pub fn eval_expr(expr: &Exp) -> Result<LuaValue> {
    match *expr {
        Exp::Nil => Ok(LuaValue::Nil),
        Exp::Bool(val) => Ok(LuaValue::Boolean(val)),
        Exp::Num(val) => Ok(match val {
            nom_lua53::num::Numeral::Float(fl) => LuaValue::Float(fl),
            nom_lua53::num::Numeral::Int(i) => LuaValue::Integer(i),
        }),
        Exp::BinExp(ref left, ref op, ref right) => operator::eval_binary_expr(left, right, &op),
        Exp::Str(ref s) => Ok(LuaValue::Str(lit_to_string(s))),
        _ => Ok(LuaValue::Nil),
    }
}
