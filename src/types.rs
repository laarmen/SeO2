use std::collections::HashMap;
use std::rc::Rc;
use std::hash::{Hash, Hasher};

#[derive(Debug,Eq)]
pub struct LuaTable {
    ref_id: usize,
    map: HashMap<LuaValue, LuaValue>,
    vector: HashMap<LuaValue, LuaValue>,
}

impl PartialEq for LuaTable {
    fn eq(&self, other: &LuaTable) -> bool {
        return self.ref_id == other.ref_id
    } 
}

impl Hash for LuaTable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ref_id.hash(state);
    }
}

#[derive(Debug,PartialEq)]
pub enum LuaValue {
    Nil,
    Integer(isize),
    Float(f64),
    Boolean(bool),
    Str(String),
    Table(Rc<LuaTable>),
}

// This trait is there to say that the equality is symmetric, reflexive and transitive,
// which isn't the case for floats (NaN != NaN). However, for now these properties are
// only used in the table hashmap, and the specs say that a Lua table won't ever have NaN
// as an index, so...
// WARNING: This is gonna bite me in the rear. 
impl Eq for LuaValue { }

impl Hash for LuaValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            &LuaValue::Nil => 0.hash(state),
            &LuaValue::Integer(ref i) => i.hash(state),
            &LuaValue::Float(ref f) => f.to_bits().hash(state),
            &LuaValue::Boolean(ref b) => b.hash(state),
            &LuaValue::Str(ref s) => s.hash(state),
            &LuaValue::Table(ref t) => t.hash(state),
        }
    }
}
