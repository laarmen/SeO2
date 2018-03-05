use types::LuaTable;
use nom_lua53::ExpSuffix;
use nom_lua53::ExpOrVarName;
use super::{eval_expr, LuaState, LuaValue, Result};

use LuaError::*;
use var_to_string;

pub fn eval_prefix_expr(
    prefix: &ExpOrVarName,
    suffix: &[ExpSuffix],
    ctx: &LuaState,
) -> Result<LuaValue> {
    let resolution = resolve_prefix_expr(prefix, suffix, ctx)?;
    return Ok(resolution.environment.get(&resolution.index).clone());
}

#[derive(Debug)]
pub struct Assignment {
    pub environment: LuaTable,
    pub index: LuaValue,
}

pub fn resolve_prefix_expr_rec(
    current: LuaValue,
    suffixes: &[ExpSuffix],
    ctx: &LuaState,
) -> Result<Assignment> {
    if suffixes.is_empty() {
        return Err(OtherError("Shouldn't happen".to_owned()));
    }
    if suffixes.len() == 1 {
        let suffix = suffixes.first();
        match suffix.unwrap() {
            &ExpSuffix::FuncCall(_) => return Err(OtherError("Shouldn't happen".to_owned())),
            &ExpSuffix::TableDot(ref name) => if let LuaValue::Table(t) = current {
                return Ok(Assignment {
                    environment: t,
                    index: LuaValue::Str(var_to_string(name)),
                });
            } else {
                return Err(TypeError("Not indexable".to_owned()));
            },
            &ExpSuffix::TableIdx(ref exp) => if let LuaValue::Table(t) = current {
                return Ok(Assignment {
                    environment: t,
                    index: eval_expr(exp, ctx)?,
                });
            } else {
                return Err(TypeError("Not indexable".to_owned()));
            },
        }
    }
    let suffix = suffixes.first().unwrap();
    let next = match suffix {
        &ExpSuffix::FuncCall(_) => return Err(NotImplementedError),
        &ExpSuffix::TableDot(ref name) => if let LuaValue::Table(t) = current {
            t.get_string(var_to_string(name))
        } else {
            return Err(TypeError("Not indexable".to_owned()));
        },
        &ExpSuffix::TableIdx(ref exp) => if let LuaValue::Table(t) = current {
            t.get(&eval_expr(exp, ctx)?)
        } else {
            return Err(TypeError("Not indexable".to_owned()));
        },
    };
    return resolve_prefix_expr_rec(next, &suffixes[1..], ctx);
}

pub fn resolve_prefix_expr(
    prefix: &ExpOrVarName,
    suffixes: &[ExpSuffix],
    ctx: &LuaState,
) -> Result<Assignment> {
    let mut suffixes = suffixes.to_vec();
    let initial_root: LuaValue = match prefix {
        &ExpOrVarName::VarName(ref name) => {
            suffixes.insert(0, ExpSuffix::TableDot(name.clone()));
            let name = var_to_string(name);
            if let Some(scope) = ctx.resolve_name(&name) {
                LuaValue::Table(scope.clone())
            } else {
                let env = "_ENV".to_owned();
                ctx.resolve_name(&env).unwrap().get_string(env)
            }
        }
        &ExpOrVarName::Exp(ref e) => eval_expr(&e, ctx)?,
    };
    resolve_prefix_expr_rec(initial_root, &suffixes, ctx)
}
