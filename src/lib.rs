extern crate nom_lua53;

use nom_lua53::{parse_all, ParseResult, Statement, Exp};
use nom_lua53::op::BinOp;

#[derive(PartialEq,Debug)]
pub enum LuaValue {
    Nil,
    Integer(isize),
    Float(f64),
    Boolean(bool),
    Str(String),
}


#[derive(PartialEq,Eq,Debug)]
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

    use nom_lua53::{parse_all, ParseResult, Statement, Exp};
    use nom_lua53::num::Numeral;
    use nom_lua53::op::BinOp;


    #[test]
    fn test_addition() {
        // 1. + -1. == 0.
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(1.0))), &Box::new(Exp::Num(Numeral::Float(-1.0))), &BinOp::Plus).unwrap();
        assert_eq!(res, LuaValue::Float(0.));

        // 1 + -1. == 0.
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(1))), &Box::new(Exp::Num(Numeral::Float(-1.0))), &BinOp::Plus).unwrap();
        assert_eq!( res, LuaValue::Float(0.));
        // 1 + 3 == 4
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(1))), &Box::new(Exp::Num(Numeral::Int(3))), &BinOp::Plus).unwrap();
        assert_eq!(res, LuaValue::Integer(4));
    }

    #[test]
    fn test_arithmetic_types() {
        eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(1))), &Box::new(Exp::Num(Numeral::Float(-1.0))), &BinOp::Plus).unwrap();

        eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(1.))), &Box::new(Exp::Num(Numeral::Int(-1))), &BinOp::Plus).unwrap();

        let res = eval_binary_expr(&Box::new(Exp::Bool(true)), &Box::new(Exp::Num(Numeral::Int(-1))), &BinOp::Plus).unwrap_err();
        assert!(match res { TypeError(_) => true, _ => false })
    }

    #[test]
    fn test_substraction() {
        // 1. - -1. == 2.
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(1.0))), &Box::new(Exp::Num(Numeral::Float(-1.0))), &BinOp::Minus).unwrap();
        assert_eq!(res, LuaValue::Float(2.));

        // 1 - -1. == 2.
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(1))), &Box::new(Exp::Num(Numeral::Float(-1.0))), &BinOp::Minus).unwrap();
        assert_eq!(res, LuaValue::Float(2.));

        // 1 - 3 == -2
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(1))), &Box::new(Exp::Num(Numeral::Int(3))), &BinOp::Minus).unwrap();
        assert_eq!(res, LuaValue::Integer(-2));
    }

    #[test]
    fn test_multiplication() {
        // 1.5 * 2.5. == 3.75.
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(1.5))), &Box::new(Exp::Num(Numeral::Float(2.5))), &BinOp::Mul).unwrap();
        assert_eq!(res, LuaValue::Float(3.75));

        // -2 * 1.25 == -2.5.
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(-2))), &Box::new(Exp::Num(Numeral::Float(1.25))), &BinOp::Mul).unwrap();
        assert_eq!(res, LuaValue::Float(-2.5));

        // 10 * 3 == 30
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(10))), &Box::new(Exp::Num(Numeral::Int(3))), &BinOp::Mul).unwrap();
        assert_eq!(res, LuaValue::Integer(30));
    }

    #[test]
    fn test_division() {
        // 1.5 / 0.5. == 3.0
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(1.5))), &Box::new(Exp::Num(Numeral::Float(0.5))), &BinOp::Div).unwrap();
        assert_eq!(res, LuaValue::Float(3.));

        // 3 / -2. == -1.5
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(3))), &Box::new(Exp::Num(Numeral::Float(-2.0))), &BinOp::Div).unwrap();
        assert_eq!(res, LuaValue::Float(-1.5));

        // 3 / 2 == 1.5
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(3))), &Box::new(Exp::Num(Numeral::Int(2))), &BinOp::Div).unwrap();
        assert_eq!(res, LuaValue::Float(1.5));
    }

    #[test]
    fn test_division_null_error() {
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(1))), &Box::new(Exp::Num(Numeral::Int(0))), &BinOp::Div).unwrap_err();
        assert!(match res { ArithmeticError(_) => true, _ => false });

        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(1.))), &Box::new(Exp::Num(Numeral::Float(0.))), &BinOp::Div).unwrap_err();
        assert!(match res { ArithmeticError(_) => true, _ => false });
    }

    #[test]
    fn test_intdiv() {
        // 1.5 // 0.5. == 3
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(1.5))), &Box::new(Exp::Num(Numeral::Float(0.5))), &BinOp::IntDiv).unwrap();
        assert_eq!(res, LuaValue::Integer(3));

        // 3 // -2. == -2
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(3))), &Box::new(Exp::Num(Numeral::Float(-2.0))), &BinOp::IntDiv).unwrap();
        assert_eq!(res, LuaValue::Integer(-2));

        // 3 // 2 == 1
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(3))), &Box::new(Exp::Num(Numeral::Int(2))), &BinOp::IntDiv).unwrap();
        assert_eq!(res, LuaValue::Integer(1));
    }
}
