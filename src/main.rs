mod cmb;
use std::{collections::HashMap, io::stdin, process::exit};

use clap::{ArgAction, Parser};

use crate::cmb::{Defs, Expr};
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

#[derive(Parser)]
struct Args {
    #[arg(short='B',action=ArgAction::SetTrue,default_value_t=false)]
    B: bool,
    #[arg(short='S',action=ArgAction::SetTrue,default_value_t=false)]
    S: bool,
    #[arg(short='K',action=ArgAction::SetTrue,default_value_t=false)]
    K: bool,
    #[arg(short='W',action=ArgAction::SetTrue,default_value_t=false)]
    W: bool,
    #[arg(short='C',action=ArgAction::SetTrue,default_value_t=false)]
    C: bool,
    #[arg(short, long)]
    trace: bool,
}

impl Args {
    fn to_defs(&self) -> Defs {
        let mut out = HashMap::new();
        if self.B {
            out.insert('B', Expr::B0);
        }
        if self.C {
            out.insert('C', Expr::C0);
        }
        if self.W {
            out.insert('W', Expr::W0);
        }
        if !self.K {
            out.insert('K', Expr::K0);
        }
        if !self.S {
            out.insert('S', Expr::S0);
        }
        out
    }
}

fn main() {
    let args = Args::parse();
    let mut sk_expr = String::new();
    println!("Input SK combinatorial expression:");
    if let Err(_) = stdin().read_line(&mut sk_expr) {
        ioerror()
    }
    let for_parse = String::from(&sk_expr[0..sk_expr.len() - 1]);
    let parsed = Expr::parse(&for_parse, &args.to_defs(), args.trace);
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
