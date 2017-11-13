extern crate nom_lua53;

use nom_lua53::{parse_all, ParseResult, Statement};
use nom_lua53::name::VarName;

use std::rc::Rc;

mod expression;
mod types;

#[derive(PartialEq,Eq,Debug)]
pub enum LuaError {
    TypeError(String),
    IndexError(String),
    ArithmeticError(String),
    OtherError(String),
    NotImplementedError,
}

type Result<T> = std::result::Result<T, LuaError>;

pub fn var_to_string(var: &VarName) -> String {
    String::from_utf8_lossy(var.0).to_string()
}

pub fn parse_statement(stmt: &Statement, ctx: &mut types::LuaState) -> Result<()>{
    match stmt {
        &Statement::LVarAssign(ref ass) => {
            let values = ass.vals.as_ref().expect("There should be some values. Why isn't there any value?!");
            for (var, val) in ass.vars.iter().zip(values.iter()) {
                let computed_value = expression::eval_expr(val, &ctx);
                let local_scope = ctx.get_mutable_local_scope().unwrap();
                Rc::get_mut(local_scope).unwrap().insert(var_to_string(var), computed_value.unwrap());
            }
        }
        &Statement::Assignment(ref ass) => {
            println!("Assigning {:?} to {:?}", ass.vals, ass.vars);
        }
        _ => {}
    }
    return Ok(());
}

pub fn eval_file(input: &[u8]) -> Result<()>{
    let mut ctx = types::LuaState::new();
    match parse_all(input) {
        ParseResult::Done(blk) => {
            for stmt in blk.stmts {
                println!("{:?}", stmt);
                parse_statement(&stmt, &mut ctx)?;
            }
            println!("{:?}", ctx.get_local_scope());
        }

        ParseResult::Error(rest, ss) => {
            println!("Error. statements == {:#?}", ss);
            println!("rest == '{}'", String::from_utf8_lossy(rest));
        }
    };
    return Ok(());
}
