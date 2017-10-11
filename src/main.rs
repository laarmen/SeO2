extern crate nom_lua53;

use std::io::{self, Read};
use nom_lua53::{parse_all, ParseResult, Statement, Exp};
use nom_lua53::op::BinOp;

#[derive(Debug)]
enum LuaValue {
    Nil,
    Integer(isize),
    Float(f64),
    Boolean(bool),
    Str(String),
}

// What do we say? We say "Merci Basile!"
fn lit_to_string(string: &nom_lua53::string::StringLit) -> String {
    return String::from_utf8_lossy(&(string.0)).to_string();
}

fn eval_addition(left_op: &Box<Exp>, right_op: &Box<Exp>, inverse: bool) -> LuaValue {
    let left_op = eval_expr(&left_op);
    let right_op = eval_expr(&right_op);

    match left_op {
        LuaValue::Integer(i) => match right_op {
            LuaValue::Integer(j) => if inverse { LuaValue::Integer(i-j) } else { LuaValue::Integer(i+j) },
            LuaValue::Float(j) => if inverse { LuaValue::Float((i as f64)-j) } else { LuaValue::Float((i as f64)+j) },
            _ => panic!("Trying to add non-numeric stuff.")
        },
        LuaValue::Float(i) => match right_op {
            LuaValue::Integer(j) => if inverse { LuaValue::Float(i-(j as f64)) } else { LuaValue::Float(i+(j as f64)) },
            LuaValue::Float(j) => if inverse { LuaValue::Float(i-j) } else { LuaValue::Float(i+j) },
            _ => panic!("Trying to add non-numeric stuff.")
        },
        _ => panic!("Trying to add non-numeric stuff.")
    }
}

fn eval_binary_expr(left_op: &Box<Exp>, right_op: &Box<Exp>, operator: &BinOp) -> LuaValue
{
    // We cannot yet evaluate the operands as some binary operators are used to shortcut
    // evaluation.
    match *operator {
        BinOp::Plus => eval_addition(left_op, right_op, false),
        BinOp::Minus => eval_addition(left_op, right_op, true),
        _ => LuaValue::Nil
    }
}

fn eval_expr(expr: &Exp) -> LuaValue {
    match *expr {
        Exp::Nil => LuaValue::Nil,
        Exp::Bool(val) => LuaValue::Boolean(val),
        Exp::Num(val) => match val {
            nom_lua53::num::Numeral::Float(fl) => LuaValue::Float(fl),
            nom_lua53::num::Numeral::Int(i) => LuaValue::Integer(i),
        },
        Exp::BinExp(ref left, ref op, ref right) => eval_binary_expr(left, right, &op),
        Exp::Str(ref s) => LuaValue::Str(lit_to_string(s)),
        _ => LuaValue::Nil,
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
                            println!("Assigning {:?} to {:?}", computed_value, var);
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
