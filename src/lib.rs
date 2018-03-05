extern crate nom_lua53;

use nom_lua53::{parse_all, ParseResult, Statement};
use nom_lua53::name::VarName;

mod expression;
mod types;

#[derive(PartialEq, Eq, Debug, Clone)]
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
                local_scope.set_string(var_to_string(var), &computed_value.unwrap());
            }
        }
        &Statement::Assignment(ref ass) => {
            let mut values = Vec::with_capacity(ass.vals.len());
            for exp in ass.vals.iter() {
                values.push(expression::eval_expr(&exp, &ctx)?);
            }

            for (prefexp, val) in ass.vars.iter().zip(values.drain(..)) {
                let assignment = expression::prefixexp::resolve_prefix_expr(
                    &prefexp.prefix,
                    &prefexp.suffix_chain,
                    ctx,
                )?;
                assignment.environment.set(&assignment.index, &val)?;
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
            println!("{:?}", ctx);
        }

        ParseResult::Error(rest, ss) => {
            println!("Error. statements == {:#?}", ss);
            println!("rest == '{}'", String::from_utf8_lossy(rest));
        }
    };
    return Ok(());
}
