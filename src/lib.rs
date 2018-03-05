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

pub fn parse_statement(stmt: &Statement, ctx: &mut types::LuaState) -> Result<()> {
    match stmt {
        &Statement::LVarAssign(ref ass) => {
            let values = ass.vals
                .as_ref()
                .expect("There should be some values. Why isn't there any value?!");
            for (var, val) in ass.vars.iter().zip(values.iter()) {
                let computed_value = expression::eval_expr(val, &ctx);
                let local_scope = ctx.get_mutable_local_scope().unwrap();
                Rc::get_mut(local_scope).unwrap().insert(var_to_string(var), computed_value.unwrap());
            }
        }
        &Statement::Assignment(ref ass) => {
            let mut values = Vec::with_capacity(ass.vals.len());
            for exp in ass.vals.iter() {
                values.push(expression::eval_expr(&exp, &ctx)?);
            }

            for (prefexp, val) in ass.vars.iter().zip(values.drain(..)) {
                if prefexp.suffix_chain.is_empty() {
                    match prefexp.prefix {
                        nom_lua53::ExpOrVarName::Exp(_) => return Err(LuaError::OtherError("This affectation doesn't make much sense...".to_owned())),
                        nom_lua53::ExpOrVarName::VarName(var) => {
                            let var = var_to_string(&var);
                            if let Some(scope) = ctx.resolve_name_mut(&var) {
                                Rc::get_mut(scope).unwrap().insert(var, val);
                            }
                        }
                    }

                }
            }
            println!("Assigning {:?} to {:?}", ass.vals, ass.vars);
        }
        _ => {}
    }
    return Ok(());
}

pub fn eval_file(input: &[u8]) -> Result<()> {
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
