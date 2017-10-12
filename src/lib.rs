extern crate nom_lua53;

use nom_lua53::{parse_all, ParseResult, Statement, Exp};
use nom_lua53::op::BinOp;

#[derive(Debug)]
pub enum LuaValue {
    Nil,
    Integer(isize),
    Float(f64),
    Boolean(bool),
    Str(String),
}


#[derive(Debug)]
pub enum LuaError {
    TypeError(String),
    ArithmeticError(String),
}

use LuaError::*;

type Result<T> = std::result::Result<T, LuaError>;

// What do we say? We say "Merci Basile!"
fn lit_to_string(string: &nom_lua53::string::StringLit) -> String {
    return String::from_utf8_lossy(&(string.0)).to_string();
}

fn eval_arithmetic(left_op: &Box<Exp>, right_op: &Box<Exp>,
                   integer: fn(isize, isize) -> Result<LuaValue>,
                   float: fn(f64, f64) -> Result<LuaValue>) -> Result<LuaValue> {

    let left_op = eval_expr(&left_op)?;
    let right_op = eval_expr(&right_op)?;

    match left_op {
        LuaValue::Integer(i) => match right_op {
            LuaValue::Integer(j) => integer(i, j),
            LuaValue::Float(j) => float(i as f64, j),
            _ => Err(TypeError("Trying to do arithmetic on non-numerical values.".to_owned()))
        },
        LuaValue::Float(i) => match right_op {
            LuaValue::Integer(j) => float(i, j as f64),
            LuaValue::Float(j) => float(i, j),
            _ => Err(TypeError("Trying to do arithmetic on non-numerical values.".to_owned()))
        },
        _ => Err(TypeError("Trying to do arithmetic on non-numerical values.".to_owned()))
    }
}

fn eval_binary_expr(left_op: &Box<Exp>, right_op: &Box<Exp>, operator: &BinOp) -> Result<LuaValue> {
    // We cannot yet evaluate the operands as some binary operators are used to shortcut
    // evaluation.
    match *operator {
        BinOp::Plus => eval_arithmetic(left_op, right_op,
                                       |i, j| Ok(LuaValue::Integer(i+j)),
                                       |i, j| Ok(LuaValue::Float(i+j))),
        BinOp::Minus => eval_arithmetic(left_op, right_op,
                                        |i, j| Ok(LuaValue::Integer(i-j)),
                                        |i, j| Ok(LuaValue::Float(i-j))),
        BinOp::Mul => eval_arithmetic(left_op, right_op,
                                      |i, j| Ok(LuaValue::Integer(i*j)),
                                      |i, j| Ok(LuaValue::Float(i*j))),
        BinOp::Div => eval_arithmetic(left_op, right_op,
                                      |i, j| if j == 0 {
                                          Err(ArithmeticError("Dividing by 0".to_owned()))
                                      } else {
                                          Ok(LuaValue::Float((i as f64)/(j as f64)))
                                      },
                                      |i, j| if j == 0. {
                                          Err(ArithmeticError("Dividing by 0".to_owned()))
                                      } else {
                                          Ok(LuaValue::Float(i/j))
                                      }),
        BinOp::Mod => eval_arithmetic(left_op, right_op,
                                      |i, j| if j == 0 {
                                          Err(ArithmeticError("Dividing by 0".to_owned()))
                                      } else {
                                          Ok(LuaValue::Integer(i % j))
                                      },
                                      |i, j| if j == 0. {
                                          Err(ArithmeticError("Dividing by 0".to_owned()))
                                      } else {
                                          Ok(LuaValue::Float(i - (i/j).floor()*j))
                                      }),
        BinOp::IntDiv => eval_arithmetic(left_op, right_op,
                                         |i, j| if j == 0 {
                                             Err(ArithmeticError("Dividing by 0".to_owned()))
                                         } else {
                                             Ok(LuaValue::Integer(i/j))
                                         },
                                         |i, j| if j == 0. {
                                             Err(ArithmeticError("Dividing by 0".to_owned()))
                                         } else {
                                             Ok(LuaValue::Integer((i/j).floor() as isize))
                                         }),
        BinOp::Pow => eval_arithmetic(left_op, right_op,
                                         |i, j| Ok(LuaValue::Float((i as f64).powf(j as f64))),
                                         |i, j| Ok(LuaValue::Float(i.powf(j)))),
        _ => Ok(LuaValue::Nil)
    }
}

fn eval_expr(expr: &Exp) -> Result<LuaValue> {
    match *expr {
        Exp::Nil => Ok(LuaValue::Nil),
        Exp::Bool(val) => Ok(LuaValue::Boolean(val)),
        Exp::Num(val) => Ok(match val {
            nom_lua53::num::Numeral::Float(fl) => LuaValue::Float(fl),
            nom_lua53::num::Numeral::Int(i) => LuaValue::Integer(i),
        }),
        Exp::BinExp(ref left, ref op, ref right) => eval_binary_expr(left, right, &op),
        Exp::Str(ref s) => Ok(LuaValue::Str(lit_to_string(s))),
        _ => Ok(LuaValue::Nil),
    }
}

pub fn eval_file(input: &[u8]) {
    match parse_all(input) {
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mytest() {
    }
}
