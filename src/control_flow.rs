use nom_lua53;
use nom_lua53::stat_expr_types::Block;
use types::LuaState;
use expression;
use std;
use super::{Result, parse_statement};
use super::types::LuaValue;

pub enum FlowDisruption {
    None,
    Return(Vec<LuaValue>),
    Break,
}

pub fn parse_block(block: &Block, ctx: &mut LuaState) -> Result<FlowDisruption> {
        for stmt in block.stmts.iter() {
            parse_statement(&stmt, ctx)?;
        };
        if let Some(ref expressions) = block.ret_stmt {
            let mut results = std::vec::Vec::with_capacity(expressions.len());
            for exp in expressions {
                results.push(expression::eval_expr(exp, ctx)?)
            }
            Ok(FlowDisruption::Return(results))
        } else {
            Ok(FlowDisruption::None)
        }
}
pub fn exec_if_then_else(ite: &nom_lua53::IfThenElse, ctx: &mut LuaState) -> Result<FlowDisruption> {
    if expression::boolean_coercion(&expression::eval_expr(&ite.cond, ctx)?) {
        parse_block(&ite.then_blk, ctx)
    } else {
        match ite.elseifs.iter().fold(None,
         |current, &(ref exp, ref block)|
          if current.is_none() {
              match expression::eval_expr(&exp, ctx) {
                  Ok(_) => Some(parse_block(&block, ctx)),
                  Err(err) => Some(Err(err))
              }
          } else { current }) {
              Some(val) => val,
              None => match &ite.else_blk {
                  &Some(ref blk) => parse_block(blk, ctx),
                  &None => Ok(FlowDisruption::None)
              }
          }
    }
}