extern crate seo2;

use std::io::{self, Read};

fn main() {
    let mut input = Vec::new();
    io::stdin()
        .read_to_end(&mut input)
        .expect("Couldn't read from stdin");

    println!("{:?}", seo2::eval_file(&input));
}
