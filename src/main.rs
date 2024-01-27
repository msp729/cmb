#![warn(
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic,
    clippy::nursery
)]

mod cmb;
use std::{
    collections::HashMap,
    fs::File,
    io::{stdin, Read},
    path::PathBuf,
    process::exit,
    rc::Rc,
};

use clap::{ArgAction, Parser, Subcommand};

use rustyline::DefaultEditor;

use crate::cmb::{assignment, Defs, Expr};
fn ioerror() {
    eprintln!("I/O error :(");
    exit(1)
}

#[derive(Parser)]
#[allow(non_snake_case, clippy::struct_excessive_bools)]
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
    #[arg(short='I',action=ArgAction::SetTrue,default_value_t=false)]
    I: bool,
    #[arg(short, long)]
    trace: bool,
    #[arg(short, long)]
    expr: Option<String>,
    #[arg()]
    file: Option<PathBuf>,
    #[command(subcommand)]
    mode: Option<Modes>,
}

#[derive(Subcommand)]
enum Modes {
    Interpeter,
    LineFilter,
    TextFilter,
}

impl Args {
    fn to_defs(&self) -> Defs {
        let mut out = HashMap::new();
        if self.C {
            out.insert('C', Expr::C0);
        }
        if self.W {
            out.insert('W', Expr::W0);
        }
        if self.B {
            out.insert('B', Expr::B0);
        }
        if !self.S {
            out.insert('S', Expr::S0);
        }
        if !self.K {
            out.insert('K', Expr::K0);
        }
        if self.I {
            out.insert('I', Expr::I);
        }
        out
    }
    fn comb(&self) -> String {
        let mut out = String::new();
        if self.C {
            out += "C";
        }
        if self.W {
            out += "W";
        }
        if self.B {
            out += "B";
        }
        if !self.S {
            out += "S";
        }
        if !self.K {
            out += "K";
        }
        if self.I {
            out += "I";
        }
        out
    }
}

fn main() {
    let args = Args::parse();
    match args.mode {
        None | Some(Modes::Interpeter) => {
            let mut d = args.to_defs();
            let mut sys = args.comb();
            let trace = args.trace;
            let mut rl = DefaultEditor::new().expect("Could not open input");
            loop {
                if let Some(c) = interpret(&mut d, trace, &sys, &mut rl) {
                    if sys.find(c) == None {
                        sys.push(c);
                    }
                }
            }
        }
        Some(Modes::LineFilter) => {
            let expression =
                find_expression(&args.expr, &args.file, &mut args.to_defs(), args.trace);
            let s = stdin();
            loop {
                let mut inp = String::new();
                if s.read_line(&mut inp).is_err() {
                    ioerror();
                }
                let line = Expr::parse(&inp, &HashMap::new(), args.trace);
                println!(
                    "{}",
                    if let Some(e) = line {
                        Expr::clone(&expression.apply(Rc::new(e), args.trace))
                    } else {
                        expression.clone()
                    }
                );
            }
        }
        Some(Modes::TextFilter) => {
            let expression =
                find_expression(&args.expr, &args.file, &mut args.to_defs(), args.trace);
            let mut s1 = stdin();
            let mut inp = String::new();
            if s1.read_to_string(&mut inp).is_err() {
                ioerror();
            }
            let line = Expr::parse(&inp, &HashMap::new(), args.trace);
            println!(
                "{}",
                if let Some(e) = line {
                    Expr::clone(&expression.apply(Rc::new(e), args.trace))
                } else {
                    expression
                }
            );
        }
    }
}

fn interpret(defs: &mut Defs, trace: bool, sys: &str, rl: &mut DefaultEditor) -> Option<char> {
    let line = rl
        .readline(&format!(":{sys}>"))
        .map_or_else(|_| exit(0), |v| v);
    let expr = line.trim();
    if !expr.is_empty() && expr.chars().next().expect("!str::is_empty") != '#' {
        if let Some((k, v)) = assignment(expr, defs, trace) {
            defs.insert(k, v);
            return Some(k);
        } else if let Some(e) = Expr::parse(expr, defs, trace) {
            println!(
                "Parsed `{}` of size {} into `{}` of size {}",
                expr,
                expr.len(),
                e,
                e.to_string().len()
            );
        }
    }
    None
}

fn filefromobuf(p: &Option<PathBuf>) -> Option<File> {
    if let Ok(f) = File::open(if let Some(path) = p {
        path
    } else {
        return None;
    }) {
        return Some(f);
    }
    None
}

fn validate(o: Option<Expr>) -> Expr {
    o.map_or_else(
        || {
            eprintln!("No valid expression was supplied.");
            exit(1);
        },
        |e| e,
    )
}

fn find_expression(
    expr: &Option<String>,
    possible: &Option<PathBuf>,
    defs: &mut Defs,
    trace: bool,
) -> Expr {
    let f = filefromobuf(possible);
    match (expr, f) {
        (None, None) => {
            eprintln!(
                "The filter modes must be supplied a filter to apply via the -e or -f options."
            );
            exit(1);
        }
        (Some(s), None) => validate(Expr::parse(s, defs, trace)),
        (None, Some(mut f)) => {
            let mut s = String::new();
            if f.read_to_string(&mut s).is_err() {
                ioerror();
            };
            validate(Expr::parse(&s, defs, trace))
        }
        (Some(s), Some(mut f)) => {
            let arg = Expr::parse(s, defs, trace);
            let mut body = String::new();
            if f.read_to_string(&mut body).is_err() {
                ioerror();
            };
            let file = Expr::parse_file(&body, defs, trace);
            match (arg, file) {
                (None, None) => validate(None),
                (Some(e), None) | (None, Some(e)) => e,
                (Some(e), Some(_)) => {
                    eprintln!("expr and file options are both valid, using expr option");
                    e
                }
            }
        }
    }
}
