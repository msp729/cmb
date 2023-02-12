mod cmb;
use std::{
    io::{stdin, Write},
    process::exit,
};

use crate::cmb::Expr;
#[warn(
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

fn ioerror() {
    eprintln!("I/O error :(");
    exit(1)
}

fn main() {
    let mut sk_expr = String::new();
    println!("Input SK combinatorial expression:");
    if let Err(_) = stdin().read_line(&mut sk_expr) {
        ioerror()
    }
    let for_parse = String::from(&sk_expr[0..sk_expr.len() - 1]);
    let parsed = Expr::parse(&for_parse, &Expr::SK_DEFS());
    if let Some(e) = parsed {
        println!(
            "Parsed `{}` of size {} into `{}` of size {}",
            for_parse,
            for_parse.len(),
            e,
            e.to_string().len()
        )
    } else {
        exit(2)
    }
}
