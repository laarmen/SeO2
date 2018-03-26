use nom_lua53;
use nom_lua53::Statement;
use nom_lua53::RepeatBlock;
use nom_lua53::stat_expr_types::Block;
use types::LuaState;
use expression;
use std;
use super::{Result, var_to_string};
use super::types::LuaValue;

#[derive(PartialEq)]
pub enum FlowControl {
    None,
    Return(Vec<LuaValue>),
    Break,
}

pub fn exec_statement(stmt: &Statement, ctx: &mut LuaState) -> Result<FlowControl> {
    match stmt {
        &Statement::LVarAssign(ref ass) => {
            let values = ass.vals
                .as_ref()
                .expect("There should be some values. Why isn't there any value?!");
            for (var, val) in ass.vars.iter().zip(values.iter()) {
                let computed_value = expression::eval_expr(val, &ctx);
                let local_scope = ctx.get_mutable_local_scope().unwrap();
                local_scope.set_string(var_to_string(var), &computed_value.unwrap());
            };
            Ok(FlowControl::None)
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
            Ok(FlowControl::None)
        }
        &Statement::Semicolon => {
            Ok(FlowControl::None)
        }
        &Statement::Ite(ref ite) => {
            exec_if_then_else(ite, ctx)
        }
        &Statement::While(ref blk) => {
            exec_while(blk, ctx)
        }
        &Statement::Repeat(ref blk) => {
            exec_repeat(blk, ctx)
        }
        &Statement::Break => Ok(FlowControl::Break),
        _ => {Ok(FlowControl::None)}
    }
}


pub fn exec_block(block: &Block, ctx: &mut LuaState) -> Result<FlowControl> {
    ctx.push_scope();
    let mut ret = FlowControl::None;
    for stmt in block.stmts.iter() {
        match exec_statement(&stmt, ctx)? {
            FlowControl::Break => { ret =  FlowControl::Break; break },
            FlowControl::Return(val) => { ret = FlowControl::Return(val); break },
            FlowControl::None => {}
        }
    };

    if ret != FlowControl::None {
        if let Some(ref expressions) = block.ret_stmt {
            let mut results = std::vec::Vec::with_capacity(expressions.len());
            for exp in expressions {
                results.push(expression::eval_expr(exp, ctx)?)
            }
            ret = FlowControl::Return(results)
        }
    }

    ctx.pop_scope();
    Ok(ret)
}

pub fn exec_if_then_else(ite: &nom_lua53::IfThenElse, ctx: &mut LuaState) -> Result<FlowControl> {
    if expression::boolean_coercion(&expression::eval_expr(&ite.cond, ctx)?) {
        exec_block(&ite.then_blk, ctx)
    } else {
        match ite.elseifs.iter().fold(None,
         |current, &(ref exp, ref block)|
          if current.is_none() {
              match expression::eval_expr(&exp, ctx) {
                  Ok(_) => Some(exec_block(&block, ctx)),
                  Err(err) => Some(Err(err))
              }
          } else { current }) {
              Some(val) => val,
              None => match &ite.else_blk {
                  &Some(ref blk) => exec_block(blk, ctx),
                  &None => Ok(FlowControl::None)
              }
          }
    }
}

pub fn exec_while(blk: &nom_lua53::WhileBlock, ctx: &mut LuaState) -> Result<FlowControl> {
    while expression::boolean_coercion(&expression::eval_expr(&blk.cond, ctx)?) {
        let disrupt = exec_block(&blk.block, ctx)?;
        match disrupt {
            FlowControl::Return(val) => {
                return Ok(FlowControl::Return(val))
            }
            FlowControl::Break => {
                return Ok(FlowControl::None)
            }
            FlowControl::None => {}
        }
    }
    return Ok(FlowControl::None)
}

pub fn exec_repeat(blk: &RepeatBlock, ctx: &mut LuaState) -> Result<FlowControl> {
    let mut ret = FlowControl::None;
    loop {
        ctx.push_scope();
        for stmt in blk.block.stmts.iter() {
            match exec_statement(&stmt, ctx)? {
                FlowControl::Break => { ret =  FlowControl::Break; break },
                FlowControl::Return(val) => { ret = FlowControl::Return(val); break },
                FlowControl::None => {}
            }
        };
        if ret != FlowControl::None {
            if let Some(ref expressions) = blk.block.ret_stmt {
                let mut results = std::vec::Vec::with_capacity(expressions.len());
                for exp in expressions {
                    results.push(expression::eval_expr(exp, ctx)?)
                }
                ret = FlowControl::Return(results);
            }
        }
        if ret != FlowControl::None || expression::boolean_coercion(&expression::eval_expr(&blk.cond, ctx)?) {
            ctx.pop_scope();
            break
        }
        ctx.pop_scope();
    };
    Ok(ret)
}