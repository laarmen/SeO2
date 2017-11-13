use std::collections::HashMap;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::hash::{Hash, Hasher};

use std::collections::vec_deque::VecDeque;
use std::collections::BTreeMap;

use super::{Result, LuaError};

type Scope = Rc<BTreeMap<String, LuaValue>>;

#[derive(Debug)]
pub struct LuaState {
     last_id: Cell<usize>,
     global: LuaTable,
     scope_stack: VecDeque<Scope>,
}

impl LuaState {
    pub fn new() -> LuaState {
        let global = LuaTable::new(0);
        let mut initial_scope = Rc::new(BTreeMap::new());
        Rc::get_mut(&mut initial_scope).unwrap().insert("_ENV".to_owned(), LuaValue::Table(global.clone()));

        let mut scope_stack = VecDeque::new();
        scope_stack.push_back(initial_scope);
        return LuaState {
            last_id: Cell::new(0),
            global,
            scope_stack
        }
    }

    pub fn get_ref_id(&self) -> usize {
        self.last_id.set(self.last_id.get()+1);
        return self.last_id.get();
    }
}

#[derive(Debug,Eq)]
struct CoreTable {
    pub ref_id: usize,
        pub map: RefCell<HashMap<LuaValue, LuaValue>>,
        pub vector: RefCell<Vec<LuaValue>>,
}

impl CoreTable {
}

impl PartialEq for CoreTable {
    fn eq(&self, other: &CoreTable) -> bool {
        return self.ref_id == other.ref_id
    } 
}

impl Hash for CoreTable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ref_id.hash(state);
    }
}

#[derive(Debug,PartialEq,Eq,Hash,Clone)]
pub struct LuaTable {
content: Rc<CoreTable>
}

impl LuaTable {
    pub fn new(id: usize) -> LuaTable {
        LuaTable {
             content: Rc::new(
                 CoreTable {
                      ref_id: id,
                      map: RefCell::new(HashMap::new()),
                      vector: RefCell::new(Vec::new()) }) }
    }

    pub fn with_capacity(id: usize, capacity: usize) -> LuaTable {
        LuaTable {
            content: Rc::new(
                CoreTable {
                    ref_id: id,
                    map: RefCell::new(HashMap::new()),
                    vector: RefCell::new(Vec::with_capacity(capacity))
                    })
        }
    }

    pub fn sequence_border(&self) -> usize {
        return self.content.vector.borrow().len();
    }

    pub fn set(&self, key: &LuaValue, value: &LuaValue) -> Result<()> {
        match *key {
            LuaValue::Nil => Err(LuaError::IndexError("Using nil as a table index".to_owned())),
                LuaValue::Float(f) => {
                    if f.is_nan() {
                        Err(LuaError::IndexError("Using NaN as a table index".to_owned()))
                    } else {
                        let round = f.round();
                        if round == f {
                            self.set(&LuaValue::Integer(round as isize), value)?
                        } else {
                            self.map_set(key, value)
                        };
                        Ok(())
                    }
                },
                LuaValue::Integer(i) => {
                    if i < 1 || (i as usize) > self.content.vector.borrow().len()+1 {
                        self.map_set(key, value);
                    } else {
                        self.sequence_set(i, value);
                    }
                    Ok(())
                }
            _ => {
                self.map_set(key, value);
                Ok(())
            }
        }
    }

    fn map_set(&self, key: &LuaValue, value: &LuaValue) { 
        let mut map = self.content.map.borrow_mut();
        match *value {
            LuaValue::Nil => {map.remove(key); () },
                _ =>{ map.insert(key.clone(), value.clone()); () }
        }
    }

    fn sequence_set(&self, idx: isize, value: &LuaValue) { 
        let mut seq = self.content.vector.borrow_mut();
        assert!(idx >= 1);

        let idx = (idx-1) as usize;
        if idx == seq.len() {
            seq.push(value.clone());
        }
    }
}

#[derive(Debug,PartialEq,Clone)]
pub enum LuaValue {
    Nil,
        Integer(isize),
        Float(f64),
        Boolean(bool),
        Str(String),
        Table(LuaTable)
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
