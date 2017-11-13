extern crate nom_lua53;

use nom_lua53::{parse_all, ParseResult, Statement};
use nom_lua53::name::VarName;

mod expression;
mod types;

#[derive(PartialEq,Eq,Debug)]
pub enum LuaError {
    TypeError(String),
    IndexError(String),
    ArithmeticError(String),
}

type Result<T> = std::result::Result<T, LuaError>;

pub fn var_to_string(var: &VarName) -> String {
    String::from_utf8_lossy(var.0).to_string()
}

pub fn parse_statement(stmt: &Statement, ctx: &mut types::LuaState) {
    match stmt {
        &Statement::LVarAssign(ref ass) => {
            let values = ass.vals.as_ref().expect("There should be some values. Why isn't there any value?!");
            for (var, val) in ass.vars.iter().zip(values.iter()) {
                let computed_value = expression::eval_expr(val, &ctx);
                println!("Assigning {:?} to {:?}", computed_value, var);
            }
        }
        &Statement::Assignment(ref ass) => {
            println!("Assigning {:?} to {:?}", ass.vals, ass.vars);
        }
        _ => {}
    }

}
pub fn eval_file(input: &[u8]) {
    let mut ctx = types::LuaState::new();
    match parse_all(input) {
        ParseResult::Done(blk) => {
            for stmt in blk.stmts {
                println!("{:?}", stmt);
                parse_statement(&stmt, &mut ctx)
            }
        }

        ParseResult::Error(rest, ss) => {
            println!("Error. statements == {:#?}", ss);
            println!("rest == '{}'", String::from_utf8_lossy(rest));
        }
    }
}
