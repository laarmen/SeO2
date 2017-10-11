extern crate nom_lua53;

use std::io::{self, Read};
use nom_lua53::{parse_all, ParseResult, Statement, Exp};

#[derive(Debug)]
enum LuaValue {
    Integer(isize),
    Float(f64),
    Boolean(bool),
    Str(String),
}

// What do we say? We say "Merci Basile!"
fn lit_to_string(string: &nom_lua53::string::StringLit) -> String {
    return String::from_utf8_lossy(&(string.0)).to_string();
}

fn eval_expr(expr: &Exp) -> std::option::Option<LuaValue> {
    match *expr {
        Exp::Nil => None,
        Exp::Bool(val) => Some(LuaValue::Boolean(val)),
        Exp::Num(val) => Some(match val {
            nom_lua53::num::Numeral::Float(fl) => LuaValue::Float(fl),
            nom_lua53::num::Numeral::Int(i) => LuaValue::Integer(i),
        }),
        Exp::Str(ref s) => Some(LuaValue::Str(lit_to_string(s))),
        _ => None,
    }
}

fn main() {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input).expect("Couldn't read from stdin");

    match parse_all(&input) {
        ParseResult::Done(blk) => {
            for stmt in blk.stmts {
                println!("{:?}", stmt);
                match stmt {
                    Statement::LVarAssign(ass) => {
                        let values = ass.vals.expect("There should be some values. Why isn't there any value?!");
                        for (var, val) in ass.vars.iter().zip(values.iter()) {

                            let computed_value = eval_expr(val);
                            match computed_value {
                                Some(lval) =>
                                    println!("Assigning {:?} to {:?}", lval, var),
                                _ => {}
                            }
                        }
                    }
                    Statement::Assignment(ass) => {
                        println!("Assigning {:?} to {:?}", ass.vals, ass.vars);
                    }
                    _ => {}
                }
            }
        }

        ParseResult::Error(rest, ss) => {
            println!("Error. statements == {:#?}", ss);
            println!("rest == '{}'", String::from_utf8_lossy(rest));
        }
    }
}
