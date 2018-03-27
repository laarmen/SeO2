use super::*;
use super::LuaError::*;

use nom_lua53::op::BinOp;
use nom_lua53::Exp;

fn eval_cmp_expr(
    left_op: &Box<Exp>,
    right_op: &Box<Exp>,
    nb_fn: fn(f64, f64) -> bool,
    str_fn: fn(&String, &String) -> bool,
    ctx: &LuaState,
) -> Result<LuaValue> {
    let left_op = eval_expr(&left_op, ctx)?;
    let right_op = eval_expr(&right_op, ctx)?;

    match (&left_op, &right_op) {
        (&LuaValue::Str(ref s1), &LuaValue::Str(ref s2)) => Ok(LuaValue::Boolean(str_fn(s1, s2))),
        (&LuaValue::Number(ref num1), &LuaValue::Number(ref num2)) => Ok(LuaValue::Boolean(nb_fn(num1.to_float(), num2.to_float()))),
        _ => Err(TypeError(
            format!("Trying to compare {:?} and  {:?}.", left_op, right_op).to_owned(),
        )),
    }
}

fn concatenation_operator(
    left_op: &Box<Exp>,
    right_op: &Box<Exp>,
    ctx: &LuaState,
) -> Result<LuaValue> {
    let left_op = num_coercion(eval_expr(&left_op, ctx)?);
    let right_op = num_coercion(eval_expr(&right_op, ctx)?);

    match (left_op, right_op) {
        (LuaValue::Str(s1),     LuaValue::Str(s2))      => Ok(LuaValue::Str(format!("{}{}", s1, s2))),
        (LuaValue::Number(n),   LuaValue::Str(s))       => Ok(LuaValue::Str(format!("{}{}", n.to_string(), s))),
        (LuaValue::Str(s),      LuaValue::Number(n))    => Ok(LuaValue::Str(format!("{}{}", s, n.to_string()))),
        (LuaValue::Number(n1),  LuaValue::Number(n2))   => Ok(LuaValue::Str(format!("{}{}", n1.to_string(), n2.to_string()))),
        _ => Err(TypeError(
            "Trying to do concatenation on non-string nor numerical values.".to_owned(),
        )),
    }
}

fn safe_left_shift(left: isize, right: isize) -> isize {
    if right < 0 {
        if right <= -(std::mem::size_of::<isize>() as isize) * 8 {
            0
        } else {
            left >> -right
        }
    } else {
        if right >= (std::mem::size_of::<isize>() as isize) * 8 {
            0
        } else {
            left << right
        }
    }
}

fn eval_arithmetic(
    left_op: &Box<Exp>,
    right_op: &Box<Exp>,
    integer: fn(isize, isize) -> Result<LuaValue>,
    float: fn(f64, f64) -> Result<LuaValue>,
    ctx: &LuaState,
) -> Result<LuaValue> {
    let left_op = num_coercion(eval_expr(&left_op, ctx)?);
    let right_op = num_coercion(eval_expr(&right_op, ctx)?);

    match (left_op, right_op) {
        (LuaValue::Number(num1), LuaValue::Number(num2)) => match (&num1, &num2) {
            (&Number::Int(i1), &Number::Int(i2))      => integer(i1, i2),
            _ => float(num1.to_float(), num2.to_float())
        }
        _ => Err(TypeError(
            "Trying to do arithmetic on non-numerical values.".to_owned(),
        )),
    }
}

pub fn eval_binary_expr(
    left_op: &Box<Exp>,
    right_op: &Box<Exp>,
    operator: &BinOp,
    ctx: &LuaState,
) -> Result<LuaValue> {
    // We cannot yet evaluate the operands as some binary operators are used to shortcut
    // evaluation.
    match *operator {
        BinOp::Concat => concatenation_operator(left_op, right_op, ctx),
        BinOp::Plus => eval_arithmetic(
            left_op,
            right_op,
            |i, j| Ok(LuaValue::Number(Number::Int(i + j))),
            |i, j| Ok(LuaValue::Number(Number::Float(i + j))),
            ctx,
        ),
        BinOp::Minus => eval_arithmetic(
            left_op,
            right_op,
            |i, j| Ok(LuaValue::Number(Number::Int(i - j))),
            |i, j| Ok(LuaValue::Number(Number::Float(i - j))),
            ctx,
        ),
        BinOp::Mul => eval_arithmetic(
            left_op,
            right_op,
            |i, j| Ok(LuaValue::Number(Number::Int(i * j))),
            |i, j| Ok(LuaValue::Number(Number::Float(i * j))),
            ctx,
        ),
        BinOp::Div => eval_arithmetic(
            left_op,
            right_op,
            |i, j| {
                if j == 0 {
                    Err(ArithmeticError("Dividing by 0".to_owned()))
                } else {
                    Ok(LuaValue::Number(Number::Float((i as f64) / (j as f64))))
                }
            },
            |i, j| {
                if j == 0. {
                    Err(ArithmeticError("Dividing by 0".to_owned()))
                } else {
                    Ok(LuaValue::Number(Number::Float(i / j)))
                }
            },
            ctx,
        ),
        BinOp::Mod => eval_arithmetic(
            left_op,
            right_op,
            |i, j| {
                if j == 0 {
                    Err(ArithmeticError("Dividing by 0".to_owned()))
                } else {
                    // Careful! The Lua operation is a true modulo whereas
                    // Rust follows the hardware "remainder" spec.
                    Ok(LuaValue::Number(Number::Int((i % j + j) % j)))
                }
            },
            |i, j| {
                if j == 0. {
                    Err(ArithmeticError("Dividing by 0".to_owned()))
                } else {
                    Ok(LuaValue::Number(Number::Float(i - (i / j).floor() * j)))
                }
            },
            ctx,
        ),
        BinOp::IntDiv => eval_arithmetic(
            left_op,
            right_op,
            |i, j| {
                if j == 0 {
                    Err(ArithmeticError("Dividing by 0".to_owned()))
                } else {
                    Ok(LuaValue::Number(Number::Int(i / j)))
                }
            },
            |i, j| {
                if j == 0. {
                    Err(ArithmeticError("Dividing by 0".to_owned()))
                } else {
                    Ok(LuaValue::Number(Number::Int((i / j).floor() as isize)))
                }
            },
            ctx,
        ),
        BinOp::Pow => eval_arithmetic(
            left_op,
            right_op,
            |i, j| Ok(LuaValue::Number(Number::Float((i as f64).powf(j as f64)))),
            |i, j| Ok(LuaValue::Number(Number::Float(i.powf(j)))),
            ctx,
        ),
        BinOp::BitAnd => eval_arithmetic(
            left_op,
            right_op,
            |i, j| Ok(LuaValue::Number(Number::Int(i & j))),
            // This is inefficient as there might have been some casting
            // already...
            |i, j| Ok(LuaValue::Number(Number::Int((i as isize) & (j as isize)))),
            ctx,
        ),
        BinOp::BitOr => eval_arithmetic(
            left_op,
            right_op,
            |i, j| Ok(LuaValue::Number(Number::Int(i | j))),
            // This is inefficient as there might have been some casting
            // already...
            |i, j| Ok(LuaValue::Number(Number::Int((i as isize) | (j as isize)))),
            ctx,
        ),
        BinOp::BitXor => eval_arithmetic(
            left_op,
            right_op,
            |i, j| Ok(LuaValue::Number(Number::Int(i ^ j))),
            // See BitAnd comment
            |i, j| Ok(LuaValue::Number(Number::Int((i as isize) ^ (j as isize)))),
            ctx,
        ),
        BinOp::BitShl => eval_arithmetic(
            left_op,
            right_op,
            |i, j| Ok(LuaValue::Number(Number::Int(safe_left_shift(i, j)))),
            |i, j| Ok(LuaValue::Number(Number::Int(safe_left_shift(i as isize, j as isize)))),
            ctx,
        ),
        BinOp::BitShr => eval_arithmetic(
            left_op,
            right_op,
            |i, j| Ok(LuaValue::Number(Number::Int(safe_left_shift(i, -j)))),
            |i, j| {
                Ok(LuaValue::Number(Number::Int(safe_left_shift(
                    i as isize,
                    -(j as isize),
                ))))
            },
            ctx,
        ),
        BinOp::Leq => eval_cmp_expr(left_op, right_op, |s1, s2| s1 <= s2, |i1, i2| i1 <= i2, ctx),
        BinOp::Lt => eval_cmp_expr(left_op, right_op, |s1, s2| s1 < s2, |i1, i2| i1 < i2, ctx),
        BinOp::Geq => eval_cmp_expr(left_op, right_op, |s1, s2| s1 >= s2, |i1, i2| i1 >= i2, ctx),
        BinOp::Gt => eval_cmp_expr(left_op, right_op, |s1, s2| s1 > s2, |i1, i2| i1 > i2, ctx),
        BinOp::Eq => Ok(LuaValue::Boolean(
            eval_expr(left_op, ctx)? == eval_expr(right_op, ctx)?,
        )),
        BinOp::Neq => Ok(LuaValue::Boolean(
            eval_expr(left_op, ctx)? != eval_expr(right_op, ctx)?,
        )),
        BinOp::BoolAnd => {
            let left_op = eval_expr(left_op, ctx)?;
            match left_op {
                LuaValue::Nil => Ok(LuaValue::Nil),
                LuaValue::Boolean(b) => if b {
                    eval_expr(right_op, ctx)
                } else {
                    Ok(LuaValue::Boolean(b))
                },
                _ => eval_expr(right_op, ctx),
            }
        }
        BinOp::BoolOr => {
            let left_op = eval_expr(left_op, ctx)?;
            match left_op {
                LuaValue::Nil => eval_expr(right_op, ctx),
                LuaValue::Boolean(b) => if b {
                    Ok(LuaValue::Boolean(b))
                } else {
                    eval_expr(right_op, ctx)
                },
                _ => Ok(left_op),
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
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(1.0))),
            &Box::new(Exp::Num(Numeral::Float(-1.0))),
            &BinOp::Plus,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Float(0.)));

        let res = eval_binary_expr(
            &Box::new(Exp::Str(StringLit(Cow::from(&b"1.0"[..])))),
            &Box::new(Exp::Num(Numeral::Float(-1.0))),
            &BinOp::Plus,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Float(0.)));

        // 1 + -1. == 0.
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Int(1))),
            &Box::new(Exp::Num(Numeral::Float(-1.0))),
            &BinOp::Plus,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Float(0.)));

        // "1" + 3 == 4
        let res = eval_binary_expr(
            &Box::new(Exp::Str(StringLit(Cow::from(&b"1"[..])))),
            &Box::new(Exp::Num(Numeral::Int(3))),
            &BinOp::Plus,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(4)));

        // 1 + 3 == 4
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Int(1))),
            &Box::new(Exp::Num(Numeral::Int(3))),
            &BinOp::Plus,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(4)));
    }

    #[test]
    fn test_arithmetic_types() {
        let ctx = LuaState::new();
        eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Int(1))),
            &Box::new(Exp::Num(Numeral::Float(-1.0))),
            &BinOp::Plus,
            &ctx,
        ).unwrap();

        eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(1.))),
            &Box::new(Exp::Num(Numeral::Int(-1))),
            &BinOp::Plus,
            &ctx,
        ).unwrap();

        eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(1.))),
            &Box::new(Exp::Str(StringLit(Cow::from(&b"-1"[..])))),
            &BinOp::Plus,
            &ctx,
        ).unwrap();

        let res = eval_binary_expr(
            &Box::new(Exp::Bool(true)),
            &Box::new(Exp::Num(Numeral::Int(-1))),
            &BinOp::Plus,
            &ctx,
        ).unwrap_err();
        assert!(match res {
            TypeError(_) => true,
            _ => false,
        })
    }

    #[test]
    fn test_substraction() {
        let ctx = LuaState::new();
        // 1. - -1. == 2.
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(1.0))),
            &Box::new(Exp::Num(Numeral::Float(-1.0))),
            &BinOp::Minus,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Float(2.)));

        // 1 - -1. == 2.
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Int(1))),
            &Box::new(Exp::Num(Numeral::Float(-1.0))),
            &BinOp::Minus,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Float(2.)));

        // 1 - 3 == -2
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Int(1))),
            &Box::new(Exp::Num(Numeral::Int(3))),
            &BinOp::Minus,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(-2)));
    }

    #[test]
    fn test_multiplication() {
        let ctx = LuaState::new();
        // 1.5 * 2.5. == 3.75.
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(1.5))),
            &Box::new(Exp::Num(Numeral::Float(2.5))),
            &BinOp::Mul,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Float(3.75)));

        // -2 * 1.25 == -2.5.
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Int(-2))),
            &Box::new(Exp::Num(Numeral::Float(1.25))),
            &BinOp::Mul,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Float(-2.5)));

        // 10 * 3 == 30
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Int(10))),
            &Box::new(Exp::Num(Numeral::Int(3))),
            &BinOp::Mul,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(30)));
    }

    #[test]
    fn test_division() {
        let ctx = LuaState::new();
        // 1.5 / 0.5. == 3.0
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(1.5))),
            &Box::new(Exp::Num(Numeral::Float(0.5))),
            &BinOp::Div,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Float(3.)));

        // 3 / -2. == -1.5
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Int(3))),
            &Box::new(Exp::Num(Numeral::Float(-2.0))),
            &BinOp::Div,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Float(-1.5)));

        // 3 / 2 == 1.5
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Int(3))),
            &Box::new(Exp::Num(Numeral::Int(2))),
            &BinOp::Div,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Float(1.5)));
    }

    #[test]
    fn test_intdiv() {
        let ctx = LuaState::new();
        // 1.5 // 0.5. == 3
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(1.5))),
            &Box::new(Exp::Num(Numeral::Float(0.5))),
            &BinOp::IntDiv,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(3)));

        // 3 // -2. == -2
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Int(3))),
            &Box::new(Exp::Num(Numeral::Float(-2.0))),
            &BinOp::IntDiv,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(-2)));

        // 3 // 2 == 1
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Int(3))),
            &Box::new(Exp::Num(Numeral::Int(2))),
            &BinOp::IntDiv,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(1)));
    }

    #[test]
    fn test_mod() {
        let ctx = LuaState::new();
        // 1.5 % 0.5. == 0.
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(1.5))),
            &Box::new(Exp::Num(Numeral::Float(0.5))),
            &BinOp::Mod,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Float(0.)));

        // -3.5 % 2. == 0.5
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(-3.5))),
            &Box::new(Exp::Num(Numeral::Float(2.0))),
            &BinOp::Mod,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Float(0.5)));

        // -4 % 3 == 2
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Int(-4))),
            &Box::new(Exp::Num(Numeral::Int(3))),
            &BinOp::Mod,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(2)));
    }

    #[test]
    fn test_divisions_null_error() {
        let ctx = LuaState::new();
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Int(1))),
            &Box::new(Exp::Num(Numeral::Int(0))),
            &BinOp::Div,
            &ctx,
        ).unwrap_err();
        assert!(match res {
            ArithmeticError(_) => true,
            _ => false,
        });

        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(1.))),
            &Box::new(Exp::Num(Numeral::Float(0.))),
            &BinOp::Div,
            &ctx,
        ).unwrap_err();
        assert!(match res {
            ArithmeticError(_) => true,
            _ => false,
        });

        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Int(1))),
            &Box::new(Exp::Num(Numeral::Int(0))),
            &BinOp::IntDiv,
            &ctx,
        ).unwrap_err();
        assert!(match res {
            ArithmeticError(_) => true,
            _ => false,
        });

        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(1.))),
            &Box::new(Exp::Num(Numeral::Float(0.))),
            &BinOp::IntDiv,
            &ctx,
        ).unwrap_err();
        assert!(match res {
            ArithmeticError(_) => true,
            _ => false,
        });

        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Int(1))),
            &Box::new(Exp::Num(Numeral::Int(0))),
            &BinOp::Mod,
            &ctx,
        ).unwrap_err();
        assert!(match res {
            ArithmeticError(_) => true,
            _ => false,
        });

        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(1.))),
            &Box::new(Exp::Num(Numeral::Float(0.))),
            &BinOp::Mod,
            &ctx,
        ).unwrap_err();
        assert!(match res {
            ArithmeticError(_) => true,
            _ => false,
        });
    }

    #[test]
    fn test_bitwise_and() {
        let ctx = LuaState::new();
        // 1.5 & 0.5 == 0
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(1.5))),
            &Box::new(Exp::Num(Numeral::Float(0.5))),
            &BinOp::BitAnd,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(0)));

        // 3.5 & 10 == 2
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(3.5))),
            &Box::new(Exp::Num(Numeral::Int(10))),
            &BinOp::BitAnd,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(2)));

        // IMAX & 42 == 42
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Int(isize::max_value()))),
            &Box::new(Exp::Num(Numeral::Int(42))),
            &BinOp::BitAnd,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(42)));
    }

    #[test]
    fn test_bitwise_or() {
        let ctx = LuaState::new();
        // 1.5 | 0.5 == 1
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(1.5))),
            &Box::new(Exp::Num(Numeral::Float(0.5))),
            &BinOp::BitOr,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(1)));

        // 3.5 | 10 == 2
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(3.5))),
            &Box::new(Exp::Num(Numeral::Int(10))),
            &BinOp::BitOr,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(11)));

        // IMAX | 42 == IMAX
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Int(isize::max_value()))),
            &Box::new(Exp::Num(Numeral::Int(42))),
            &BinOp::BitOr,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(isize::max_value())));
    }

    #[test]
    fn test_bitwise_xor() {
        let ctx = LuaState::new();
        // 5.5 ^ 1.5 == 4
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(5.5))),
            &Box::new(Exp::Num(Numeral::Float(1.5))),
            &BinOp::BitXor,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(4)));

        // 3.5 ^ 10 == 9
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(3.5))),
            &Box::new(Exp::Num(Numeral::Int(10))),
            &BinOp::BitXor,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(9)));

        // IMAX ^ 42 == IMAX - 42
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Int(isize::max_value()))),
            &Box::new(Exp::Num(Numeral::Int(42))),
            &BinOp::BitXor,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(isize::max_value() - 42)));
    }

    #[test]
    fn test_bitwise_shl() {
        let ctx = LuaState::new();
        // 5.5 << 1.5 == 10
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(5.5))),
            &Box::new(Exp::Num(Numeral::Float(1.5))),
            &BinOp::BitShl,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(10)));

        // 3.5 << 10 == 3072
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(3.5))),
            &Box::new(Exp::Num(Numeral::Int(10))),
            &BinOp::BitShl,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(3072)));

        // 1124 << -9 == 1
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Int(1124))),
            &Box::new(Exp::Num(Numeral::Int(-10))),
            &BinOp::BitShl,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(1)));

        // 1124 << 1024 == 0
        // Would overflow in Rust, not in Lua
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Int(1124))),
            &Box::new(Exp::Num(Numeral::Int(1024))),
            &BinOp::BitShl,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(0)));
    }

    #[test]
    fn test_cmp_leq() {
        let ctx = LuaState::new();
        // 5.5 <= 1.5 == false
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(5.5))),
            &Box::new(Exp::Num(Numeral::Float(1.5))),
            &BinOp::Leq,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Boolean(false));

        // 3.5 <= 10 == true
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(3.5))),
            &Box::new(Exp::Num(Numeral::Int(10))),
            &BinOp::Leq,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Boolean(true));

        // abc <= bcd == true
        let res = eval_binary_expr(
            &Box::new(Exp::Str(StringLit(Cow::from(&b"abc"[..])))),
            &Box::new(Exp::Str(StringLit(Cow::from(&b"bcd"[..])))),
            &BinOp::Leq,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Boolean(true));

        // abc <= bcd == true
        let res = eval_binary_expr(
            &Box::new(Exp::Str(StringLit(Cow::from(&b"abc"[..])))),
            &Box::new(Exp::Num(Numeral::Float(1.0))),
            &BinOp::Leq,
            &ctx,
        ).unwrap_err();
        assert!(match res {
            TypeError(_) => true,
            _ => false,
        });
    }

    #[test]
    fn test_bool_and() {
        let ctx = LuaState::new();
        // nil and 1.5 == nil
        let res = eval_binary_expr(
            &Box::new(Exp::Nil),
            &Box::new(Exp::Num(Numeral::Float(1.5))),
            &BinOp::BoolAnd,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Nil);

        // 3.5 and 10 == 10
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(3.5))),
            &Box::new(Exp::Num(Numeral::Int(10))),
            &BinOp::BoolAnd,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Int(10)));
    }

    #[test]
    fn test_bool_or() {
        let ctx = LuaState::new();
        // nil or 1.5 == 1.5
        let res = eval_binary_expr(
            &Box::new(Exp::Nil),
            &Box::new(Exp::Num(Numeral::Float(1.5))),
            &BinOp::BoolOr,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Float(1.5)));

        // 3.5 or 10 == 3.5
        let res = eval_binary_expr(
            &Box::new(Exp::Num(Numeral::Float(3.5))),
            &Box::new(Exp::Num(Numeral::Int(10))),
            &BinOp::BoolOr,
            &ctx,
        ).unwrap();
        assert_eq!(res, LuaValue::Number(Number::Float(3.5)));
    }
}
