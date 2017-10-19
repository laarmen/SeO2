extern crate nom_lua53;

use nom_lua53::{parse_all, ParseResult, Statement};

mod expression;

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

type Result<T> = std::result::Result<T, LuaError>;

pub fn eval_file(input: &[u8]) {
    match parse_all(input) {
        ParseResult::Done(blk) => {
            for stmt in blk.stmts {
                println!("{:?}", stmt);
                match stmt {
                    Statement::LVarAssign(ass) => {
                        let values = ass.vals.expect("There should be some values. Why isn't there any value?!");
                        for (var, val) in ass.vars.iter().zip(values.iter()) {
                            let computed_value = expression::eval_expr(val);
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
