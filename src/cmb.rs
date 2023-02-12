use std::collections::{HashMap, VecDeque};
use std::fmt::{Display, Error, Formatter};
use std::rc::Rc;

type Defs = HashMap<char, Expr>;

#[derive(Clone, Debug)]
pub enum Expr {
    Variable(char),
    LongVar(String),
    S,
    S1(Rc<Expr>),
    S2(Rc<Expr>, Rc<Expr>),
    K,
    K1(Rc<Expr>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "{}",
            match self {
                Expr::Variable(c) => c.to_string(),
                Expr::LongVar(name) => name.to_owned(),
                Expr::S => String::from("S"),
                Expr::S1(x) => format!("S{}", x.arg()),
                Expr::S2(x, y) => format!("S{}{}", x.arg(), y.arg()),
                Expr::K => String::from("K"),
                Expr::K1(x) => format!("K{}", x.arg()),
            }
        )
    }
}

#[cfg(test)]
mod parse_tests {
    use super::Expr;

    #[test]
    fn S_Combinator() {
        let to_parse = &"S".to_string();
        let result = "S".to_string();
        let parsed = Expr::parse(to_parse, &Expr::SK_DEFS()).unwrap();
        assert_eq!(parsed.to_string(), result)
    }

    #[test]
    fn S_Combinator_eval() {
        let to_parse = &"Sxyz".to_string();
        let result = "xz(yz)".to_string();
        let parsed = Expr::parse(to_parse, &Expr::SK_DEFS()).unwrap();
        assert_eq!(parsed.to_string(), result)
    }

    #[test]
    fn K_Combinator() {
        let to_parse = &"K".to_string();
        let result = "K".to_string();
        let parsed = Expr::parse(to_parse, &Expr::SK_DEFS()).unwrap();
        assert_eq!(parsed.to_string(), result)
    }

    #[test]
    fn K_Combinator_eval() {
        let to_parse = &"Kxy".to_string();
        let result = "x".to_string();
        let parsed = Expr::parse(to_parse, &Expr::SK_DEFS()).unwrap();
        assert_eq!(parsed.to_string(), result)
    }
}

impl<'a> Expr {
    fn call(&'a self, other: Rc<Expr>) -> Rc<Expr> {
        match self {
            Expr::LongVar(name) => Rc::new(Expr::LongVar(name.to_string() + &other.arg())),
            Expr::Variable(name) => Rc::new(Expr::LongVar(name.to_string() + &other.arg())),
            Expr::S => Rc::new(Expr::S1(other.clone())),
            Expr::S1(x) => Rc::new(Expr::S2(x.clone(), other.clone())),
            Expr::S2(x, y) => x.call(other.clone()).call(y.call(other)),
            Expr::K => Rc::new(Expr::K1(other.clone())),
            Expr::K1(x) => x.clone(),
        }
    }
    fn arg(&self) -> String {
        match self {
            Expr::Variable(name) => name.to_string(),
            Expr::LongVar(name) => format!("({})", name),
            Expr::S => String::from("S"),
            Expr::S1(x) => format!("(S{})", x.arg()),
            Expr::S2(x, y) => format!("(S{}{})", x.arg(), y.arg()),
            Expr::K => String::from("K"),
            Expr::K1(x) => format!("(K{})", x.arg()),
        }
    }
    pub fn parse(s: &String, d: &Defs) -> Option<Expr> {
        let mut tokens: VecDeque<Expr> = VecDeque::new();
        let mut to_ignore = 0;
        for (i, c) in s.chars().enumerate() {
            if to_ignore != 0 {
                to_ignore -= 1;
                continue;
            }
            if c == ' ' {
                continue;
            } else if c == '(' {
                let mut running = String::new();
                let mut depth = 1;
                let mut it = s.chars();
                it.nth(i);
                while depth != 0 {
                    if let Some(c2) = it.next() {
                        if c2 == ')' {
                            depth -= 1;
                        } else if c2 == '(' {
                            depth += 1;
                        }
                        if depth != 0 {
                            running += &c2.to_string();
                        }
                    } else {
                        return None;
                    }
                }
                to_ignore = running.len() + 1;
                tokens.push_back(Expr::parse(&running, d)?);
            } else if d.contains_key(&c) {
                tokens.push_back(d.get(&c)?.clone());
            } else {
                tokens.push_back(Expr::Variable(c));
            }
        }
        if tokens.len() == 0 {
            return None;
        }
        let mut out = tokens.pop_front()?;
        while !tokens.is_empty() {
            out = Expr::clone(&out.call(Rc::new(tokens.pop_front()?)));
        }
        Some(out)
    }

    pub fn SK_DEFS() -> Defs {
        HashMap::from([('S', Expr::S), ('K', Expr::K)])
    }
}
