extern crate nom_lua53;

use std::io::{self, Read};
use nom_lua53::{parse_all, ParseResult, Statement};

#[derive(Debug)]
enum LuaValue {
    Integer(isize),
    Float(f64),
    Boolean(bool),
    Str(String),
}

fn main() {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input).expect("Couldn't read from stdin");

    match parse_all(&*input) {
        ParseResult::Done(blk) => {
            for stmt in blk.stmts {
                println!("{:?}", stmt);
                match stmt {
                    Statement::LVarAssign(ass) => {
                        println!("Assigning {:?} to {:?}", ass.vals, ass.vars);
                    }
                    Statement::Assignment(ass) => {
                        println!("Assigning {:?} to {:?}", ass.vals, ass.vars);
                    }
                    _ => {}
                }
            }
        }

        ParseResult::Error(rest, ss) => {
            println!("Error. statements == {:#?}", ss);
            println!("rest == '{}'", String::from_utf8_lossy(rest));
        }
    }
}
