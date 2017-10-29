use super::*;
use super::LuaError::*;

use nom_lua53::op::BinOp;
use nom_lua53::Exp;

fn eval_cmp_expr(left_op: &Box<Exp>, right_op: &Box<Exp>,
                    nb_fn: fn(f64, f64) -> bool,
                    str_fn: fn(String, String) -> bool, ctx: &LuaState) -> Result<LuaValue> {
    let left_op = eval_expr(&left_op, ctx)?;
    let right_op = eval_expr(&right_op, ctx)?;

    match left_op {
        LuaValue::Str(s1) => match right_op {
            LuaValue::Str(s2) => Ok(LuaValue::Boolean(str_fn(s1, s2))),
            _ => Err(TypeError(format!("Trying to compare {:?} and  {:?}.", LuaValue::Str(s1), right_op).to_owned()))
        },
        LuaValue::Integer(i1) => match right_op {
            LuaValue::Integer(i2) => Ok(LuaValue::Boolean(nb_fn(i1 as f64, i2 as f64))),
            LuaValue::Float(f) => Ok(LuaValue::Boolean(nb_fn(i1 as f64, f))),
            _ => Err(TypeError(format!("Trying to compare {:?} and  {:?}.", LuaValue::Integer(i1), right_op).to_owned()))
        },
        LuaValue::Float(f1) => match right_op {
            LuaValue::Integer(i) => Ok(LuaValue::Boolean(nb_fn(f1, i as f64))),
            LuaValue::Float(f2) => Ok(LuaValue::Boolean(nb_fn(f1, f2))),
            _ => Err(TypeError(format!("Trying to compare {:?} and  {:?}.", LuaValue::Float(f1), right_op).to_owned()))
        },
        _ => Err(TypeError(format!("Trying to compare {:?} and  {:?}.", left_op, right_op).to_owned()))
    }
}

fn concatenation_operator(left_op: &Box<Exp>, right_op: &Box<Exp>, ctx: &LuaState) -> Result<LuaValue> {
    let left_op = num_coercion(eval_expr(&left_op, ctx)?);
    let right_op = num_coercion(eval_expr(&right_op, ctx)?);

    match left_op {
        LuaValue::Integer(i) => match right_op {
            LuaValue::Integer(j) => Ok(LuaValue::Str(format!("{}{}", i, j))),
            LuaValue::Float(f) => Ok(LuaValue::Str(format!("{}{}", i, f))),
            LuaValue::Str(s) => Ok(LuaValue::Str(format!("{}{}", i, s))),
            _ => Err(TypeError("Trying to do concatenation on non-string nor numerical values.".to_owned()))
        },
        LuaValue::Float(ff) => match right_op {
            LuaValue::Integer(j) => Ok(LuaValue::Str(format!("{}{}", ff, j))),
            LuaValue::Float(f) => Ok(LuaValue::Str(format!("{}{}", ff, f))),
            LuaValue::Str(s) => Ok(LuaValue::Str(format!("{}{}", ff, s))),
            _ => Err(TypeError("Trying to do concatenation on non-string nor numerical values.".to_owned()))
        },
        LuaValue::Str(ss) => match right_op {
            LuaValue::Integer(j) => Ok(LuaValue::Str(format!("{}{}", ss, j))),
            LuaValue::Float(f) => Ok(LuaValue::Str(format!("{}{}", ss, f))),
            LuaValue::Str(s) => Ok(LuaValue::Str(format!("{}{}", ss, s))),
            _ => Err(TypeError("Trying to do concatenation on non-string nor numerical values.".to_owned()))
        },
        _ => Err(TypeError("Trying to do concatenation on non-string nor numerical values.".to_owned()))
    }
}

fn safe_left_shift(left: isize, right: isize) -> isize {
    if right < 0 {
        if right <= -(std::mem::size_of::<isize>() as isize) * 8 {
            0
        } else {
            left >> -right
        }
    }
    else {
        if right >= (std::mem::size_of::<isize>() as isize) * 8 {
            0
        } else {
            left << right
        }
    }
}

fn eval_arithmetic(left_op: &Box<Exp>, right_op: &Box<Exp>,
                   integer: fn(isize, isize) -> Result<LuaValue>,
                   float: fn(f64, f64) -> Result<LuaValue>, ctx: &LuaState) -> Result<LuaValue> {

    let left_op = num_coercion(eval_expr(&left_op, ctx)?);
    let right_op = num_coercion(eval_expr(&right_op, ctx)?);

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

pub fn eval_binary_expr(left_op: &Box<Exp>, right_op: &Box<Exp>, operator: &BinOp, ctx: &LuaState) -> Result<LuaValue> {
    // We cannot yet evaluate the operands as some binary operators are used to shortcut
    // evaluation.
    match *operator {
        BinOp::Concat => concatenation_operator(left_op, right_op, ctx),
        BinOp::Plus => eval_arithmetic(left_op, right_op,
                                       |i, j| Ok(LuaValue::Integer(i+j)),
                                       |i, j| Ok(LuaValue::Float(i+j)), ctx),
        BinOp::Minus => eval_arithmetic(left_op, right_op,
                                        |i, j| Ok(LuaValue::Integer(i-j)),
                                        |i, j| Ok(LuaValue::Float(i-j)), ctx),
        BinOp::Mul => eval_arithmetic(left_op, right_op,
                                      |i, j| Ok(LuaValue::Integer(i*j)),
                                      |i, j| Ok(LuaValue::Float(i*j)), ctx),
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
                                      }, ctx),
        BinOp::Mod => eval_arithmetic(left_op, right_op,
                                      |i, j| if j == 0 {
                                          Err(ArithmeticError("Dividing by 0".to_owned()))
                                      } else {
                                          // Careful! The Lua operation is a true modulo whereas
                                          // Rust follows the hardware "remainder" spec.
                                          Ok(LuaValue::Integer((i % j + j) % j))
                                      },
                                      |i, j| if j == 0. {
                                          Err(ArithmeticError("Dividing by 0".to_owned()))
                                      } else {
                                          Ok(LuaValue::Float(i - (i/j).floor()*j))
                                      }, ctx),
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
                                         }, ctx),
        BinOp::Pow => eval_arithmetic(left_op, right_op,
                                      |i, j| Ok(LuaValue::Float((i as f64).powf(j as f64))),
                                      |i, j| Ok(LuaValue::Float(i.powf(j))), ctx),
        BinOp::BitAnd => eval_arithmetic(left_op, right_op,
                                      |i, j| Ok(LuaValue::Integer(i & j)),
                                      // This is inefficient as there might have been some casting
                                      // already...
                                      |i, j| Ok(LuaValue::Integer((i as isize) & (j as isize))), ctx),
        BinOp::BitOr => eval_arithmetic(left_op, right_op,
                                      |i, j| Ok(LuaValue::Integer(i | j)),
                                      // This is inefficient as there might have been some casting
                                      // already...
                                      |i, j| Ok(LuaValue::Integer((i as isize) | (j as isize))), ctx),
        BinOp::BitXor => eval_arithmetic(left_op, right_op,
                                      |i, j| Ok(LuaValue::Integer(i ^ j)),
                                      // See BitAnd comment
                                      |i, j| Ok(LuaValue::Integer((i as isize) ^ (j as isize))), ctx),
        BinOp::BitShl => eval_arithmetic(left_op, right_op,
                                      |i, j| Ok(LuaValue::Integer(safe_left_shift(i, j))),
                                      |i, j| Ok(LuaValue::Integer(safe_left_shift(i as isize, j as isize))), ctx),
        BinOp::BitShr => eval_arithmetic(left_op, right_op,
                                      |i, j| Ok(LuaValue::Integer(safe_left_shift(i, -j))),
                                      |i, j| Ok(LuaValue::Integer(safe_left_shift(i as isize, -(j as isize)))), ctx),
        BinOp::Leq => eval_cmp_expr(left_op, right_op, |s1, s2| s1 <= s2, |i1, i2| i1 <= i2, ctx),
        BinOp::Lt => eval_cmp_expr(left_op, right_op, |s1, s2| s1 < s2, |i1, i2| i1 < i2, ctx),
        BinOp::Geq => eval_cmp_expr(left_op, right_op, |s1, s2| s1 >= s2, |i1, i2| i1 >= i2, ctx),
        BinOp::Gt => eval_cmp_expr(left_op, right_op, |s1, s2| s1 > s2, |i1, i2| i1 > i2, ctx),
        BinOp::Eq => Ok(LuaValue::Boolean(eval_expr(left_op, ctx)? == eval_expr(right_op, ctx)?)),
        BinOp::Neq => Ok(LuaValue::Boolean(eval_expr(left_op, ctx)? != eval_expr(right_op, ctx)?)),
        BinOp::BoolAnd => {
            let left_op = eval_expr(left_op, ctx)?;
            match left_op {
                LuaValue::Nil => Ok(LuaValue::Nil),
                LuaValue::Boolean(b) => if b { eval_expr(right_op, ctx) } else { Ok(LuaValue::Boolean(b)) },
                _ => eval_expr(right_op, ctx)
            }
        },
        BinOp::BoolOr => {
            let left_op = eval_expr(left_op, ctx)?;
            match left_op {
                LuaValue::Nil => eval_expr(right_op, ctx),
                LuaValue::Boolean(b) => if b { Ok(LuaValue::Boolean(b)) } else { eval_expr(right_op, ctx) },
                _ => Ok(left_op)
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
    use nom_lua53::op::BinOp;

    use std::borrow::Cow;


    #[test]
    fn test_addition() {
        let ctx = LuaState::new();
        // 1. + -1. == 0.
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(1.0))), &Box::new(Exp::Num(Numeral::Float(-1.0))), &BinOp::Plus, &ctx).unwrap();
        assert_eq!(res, LuaValue::Float(0.));

        let res = eval_binary_expr(&Box::new(Exp::Str(StringLit(Cow::from(&b"1.0"[..])))), &Box::new(Exp::Num(Numeral::Float(-1.0))), &BinOp::Plus, &ctx).unwrap();
        assert_eq!(res, LuaValue::Float(0.));

        // 1 + -1. == 0.
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(1))), &Box::new(Exp::Num(Numeral::Float(-1.0))), &BinOp::Plus, &ctx).unwrap();
        assert_eq!( res, LuaValue::Float(0.));

        // "1" + 3 == 4
        let res = eval_binary_expr(&Box::new(Exp::Str(StringLit(Cow::from(&b"1"[..])))), &Box::new(Exp::Num(Numeral::Int(3))), &BinOp::Plus, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(4));

        // 1 + 3 == 4
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(1))), &Box::new(Exp::Num(Numeral::Int(3))), &BinOp::Plus, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(4));
    }

    #[test]
    fn test_arithmetic_types() {
        let ctx = LuaState::new();
        eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(1))), &Box::new(Exp::Num(Numeral::Float(-1.0))), &BinOp::Plus, &ctx).unwrap();

        eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(1.))), &Box::new(Exp::Num(Numeral::Int(-1))), &BinOp::Plus, &ctx).unwrap();

        eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(1.))), &Box::new(Exp::Str(StringLit(Cow::from(&b"-1"[..])))), &BinOp::Plus, &ctx).unwrap();

        let res = eval_binary_expr(&Box::new(Exp::Bool(true)), &Box::new(Exp::Num(Numeral::Int(-1))), &BinOp::Plus, &ctx).unwrap_err();
        assert!(match res { TypeError(_) => true, _ => false })
    }

    #[test]
    fn test_substraction() {
        let ctx = LuaState::new();
        // 1. - -1. == 2.
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(1.0))), &Box::new(Exp::Num(Numeral::Float(-1.0))), &BinOp::Minus, &ctx).unwrap();
        assert_eq!(res, LuaValue::Float(2.));

        // 1 - -1. == 2.
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(1))), &Box::new(Exp::Num(Numeral::Float(-1.0))), &BinOp::Minus, &ctx).unwrap();
        assert_eq!(res, LuaValue::Float(2.));

        // 1 - 3 == -2
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(1))), &Box::new(Exp::Num(Numeral::Int(3))), &BinOp::Minus, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(-2));
    }

    #[test]
    fn test_multiplication() {
        let ctx = LuaState::new();
        // 1.5 * 2.5. == 3.75.
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(1.5))), &Box::new(Exp::Num(Numeral::Float(2.5))), &BinOp::Mul, &ctx).unwrap();
        assert_eq!(res, LuaValue::Float(3.75));

        // -2 * 1.25 == -2.5.
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(-2))), &Box::new(Exp::Num(Numeral::Float(1.25))), &BinOp::Mul, &ctx).unwrap();
        assert_eq!(res, LuaValue::Float(-2.5));

        // 10 * 3 == 30
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(10))), &Box::new(Exp::Num(Numeral::Int(3))), &BinOp::Mul, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(30));
    }

    #[test]
    fn test_division() {
        let ctx = LuaState::new();
        // 1.5 / 0.5. == 3.0
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(1.5))), &Box::new(Exp::Num(Numeral::Float(0.5))), &BinOp::Div, &ctx).unwrap();
        assert_eq!(res, LuaValue::Float(3.));

        // 3 / -2. == -1.5
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(3))), &Box::new(Exp::Num(Numeral::Float(-2.0))), &BinOp::Div, &ctx).unwrap();
        assert_eq!(res, LuaValue::Float(-1.5));

        // 3 / 2 == 1.5
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(3))), &Box::new(Exp::Num(Numeral::Int(2))), &BinOp::Div, &ctx).unwrap();
        assert_eq!(res, LuaValue::Float(1.5));
    }

    #[test]
    fn test_intdiv() {
        let ctx = LuaState::new();
        // 1.5 // 0.5. == 3
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(1.5))), &Box::new(Exp::Num(Numeral::Float(0.5))), &BinOp::IntDiv, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(3));

        // 3 // -2. == -2
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(3))), &Box::new(Exp::Num(Numeral::Float(-2.0))), &BinOp::IntDiv, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(-2));

        // 3 // 2 == 1
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(3))), &Box::new(Exp::Num(Numeral::Int(2))), &BinOp::IntDiv, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(1));
    }

    #[test]
    fn test_mod() {
        let ctx = LuaState::new();
        // 1.5 % 0.5. == 0.
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(1.5))), &Box::new(Exp::Num(Numeral::Float(0.5))), &BinOp::Mod, &ctx).unwrap();
        assert_eq!(res, LuaValue::Float(0.));

        // -3.5 % 2. == 0.5
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(-3.5))), &Box::new(Exp::Num(Numeral::Float(2.0))), &BinOp::Mod, &ctx).unwrap();
        assert_eq!(res, LuaValue::Float(0.5));

        // -4 % 3 == 2
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(-4))), &Box::new(Exp::Num(Numeral::Int(3))), &BinOp::Mod, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(2));
    }

    #[test]
    fn test_divisions_null_error() {
        let ctx = LuaState::new();
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(1))), &Box::new(Exp::Num(Numeral::Int(0))), &BinOp::Div, &ctx).unwrap_err();
        assert!(match res { ArithmeticError(_) => true, _ => false });

        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(1.))), &Box::new(Exp::Num(Numeral::Float(0.))), &BinOp::Div, &ctx).unwrap_err();
        assert!(match res { ArithmeticError(_) => true, _ => false });

        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(1))), &Box::new(Exp::Num(Numeral::Int(0))), &BinOp::IntDiv, &ctx).unwrap_err();
        assert!(match res { ArithmeticError(_) => true, _ => false });

        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(1.))), &Box::new(Exp::Num(Numeral::Float(0.))), &BinOp::IntDiv, &ctx).unwrap_err();
        assert!(match res { ArithmeticError(_) => true, _ => false });

        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(1))), &Box::new(Exp::Num(Numeral::Int(0))), &BinOp::Mod, &ctx).unwrap_err();
        assert!(match res { ArithmeticError(_) => true, _ => false });

        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(1.))), &Box::new(Exp::Num(Numeral::Float(0.))), &BinOp::Mod, &ctx).unwrap_err();
        assert!(match res { ArithmeticError(_) => true, _ => false });
    }

    #[test]
    fn test_bitwise_and() {
        let ctx = LuaState::new();
        // 1.5 & 0.5 == 0
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(1.5))), &Box::new(Exp::Num(Numeral::Float(0.5))), &BinOp::BitAnd, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(0));

        // 3.5 & 10 == 2
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(3.5))), &Box::new(Exp::Num(Numeral::Int(10))), &BinOp::BitAnd, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(2));

        // IMAX & 42 == 42
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(isize::max_value()))), &Box::new(Exp::Num(Numeral::Int(42))), &BinOp::BitAnd, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(42));
    }

    #[test]
    fn test_bitwise_or() {
        let ctx = LuaState::new();
        // 1.5 | 0.5 == 1
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(1.5))), &Box::new(Exp::Num(Numeral::Float(0.5))), &BinOp::BitOr, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(1));

        // 3.5 | 10 == 2
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(3.5))), &Box::new(Exp::Num(Numeral::Int(10))), &BinOp::BitOr, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(11));

        // IMAX | 42 == IMAX
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(isize::max_value()))), &Box::new(Exp::Num(Numeral::Int(42))), &BinOp::BitOr, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(isize::max_value()));
    }

    #[test]
    fn test_bitwise_xor() {
        let ctx = LuaState::new();
        // 5.5 ^ 1.5 == 4
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(5.5))), &Box::new(Exp::Num(Numeral::Float(1.5))), &BinOp::BitXor, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(4));

        // 3.5 ^ 10 == 9
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(3.5))), &Box::new(Exp::Num(Numeral::Int(10))), &BinOp::BitXor, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(9));

        // IMAX ^ 42 == IMAX - 42
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(isize::max_value()))), &Box::new(Exp::Num(Numeral::Int(42))), &BinOp::BitXor, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(isize::max_value() - 42));
    }

    #[test]
    fn test_bitwise_shl() {
        let ctx = LuaState::new();
        // 5.5 << 1.5 == 10
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(5.5))), &Box::new(Exp::Num(Numeral::Float(1.5))), &BinOp::BitShl, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(10));

        // 3.5 << 10 == 3072
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(3.5))), &Box::new(Exp::Num(Numeral::Int(10))), &BinOp::BitShl, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(3072));

        // 1124 << -9 == 1
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(1124))), &Box::new(Exp::Num(Numeral::Int(-10))), &BinOp::BitShl, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(1));

        // 1124 << 1024 == 0
        // Would overflow in Rust, not in Lua
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Int(1124))), &Box::new(Exp::Num(Numeral::Int(1024))), &BinOp::BitShl, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(0));
    }

    #[test]
    fn test_cmp_leq() {
        let ctx = LuaState::new();
        // 5.5 <= 1.5 == false
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(5.5))), &Box::new(Exp::Num(Numeral::Float(1.5))), &BinOp::Leq, &ctx).unwrap();
        assert_eq!(res, LuaValue::Boolean(false));

        // 3.5 <= 10 == true
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(3.5))), &Box::new(Exp::Num(Numeral::Int(10))), &BinOp::Leq, &ctx).unwrap();
        assert_eq!(res, LuaValue::Boolean(true));

        // abc <= bcd == true
        let res = eval_binary_expr(&Box::new(Exp::Str(StringLit(Cow::from(&b"abc"[..])))), &Box::new(Exp::Str(StringLit(Cow::from(&b"bcd"[..])))), &BinOp::Leq, &ctx).unwrap();
        assert_eq!(res, LuaValue::Boolean(true));

        // abc <= bcd == true
        let res = eval_binary_expr(&Box::new(Exp::Str(StringLit(Cow::from(&b"abc"[..])))), &Box::new(Exp::Num(Numeral::Float(1.0))), &BinOp::Leq, &ctx).unwrap_err();
        assert!(match res { TypeError(_) => true, _ => false });
    }

    #[test]
    fn test_bool_and() {
        let ctx = LuaState::new();
        // nil and 1.5 == nil
        let res = eval_binary_expr(&Box::new(Exp::Nil), &Box::new(Exp::Num(Numeral::Float(1.5))), &BinOp::BoolAnd, &ctx).unwrap();
        assert_eq!(res, LuaValue::Nil);

        // 3.5 and 10 == 10
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(3.5))), &Box::new(Exp::Num(Numeral::Int(10))), &BinOp::BoolAnd, &ctx).unwrap();
        assert_eq!(res, LuaValue::Integer(10));
    }

    #[test]
    fn test_bool_or() {
        let ctx = LuaState::new();
        // nil or 1.5 == 1.5
        let res = eval_binary_expr(&Box::new(Exp::Nil), &Box::new(Exp::Num(Numeral::Float(1.5))), &BinOp::BoolOr, &ctx).unwrap();
        assert_eq!(res, LuaValue::Float(1.5));

        // 3.5 or 10 == 3.5
        let res = eval_binary_expr(&Box::new(Exp::Num(Numeral::Float(3.5))), &Box::new(Exp::Num(Numeral::Int(10))), &BinOp::BoolOr, &ctx).unwrap();
        assert_eq!(res, LuaValue::Float(3.5));
    }
}