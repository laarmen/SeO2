extern crate nom_lua53;

use nom_lua53::{parse_all, ParseResult};
use nom_lua53::name::VarName;

mod expression;
mod types;
mod control_flow;

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

pub fn eval_file(input: &[u8]) -> Result<()> {
    let mut ctx = types::LuaState::new();
    match parse_all(input) {
        ParseResult::Done(blk) => {
            let _ = control_flow::exec_block(&blk, &mut ctx);
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
