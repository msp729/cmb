use std::collections::{HashMap, VecDeque};
use std::fmt::{Display, Error, Formatter};
use std::rc::Rc;

pub type Defs = HashMap<char, Expr>;
type C = Rc<Expr>; // it's short for curry

#[derive(Clone, Debug)]
pub enum Expr {
    Variable(char),
    LongVar(String),
    S0,
    S1(C),
    S2(C, C),
    K0,
    K1(C),
    W0,
    W1(C),
    C0,
    C1(C),
    C2(C, C),
    B0,
    B1(C),
    B2(C, C),
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "{}",
            match self {
                Expr::Variable(c) => c.to_string(),
                Expr::LongVar(name) => name.to_owned(),
                Expr::S0 => "S".to_string(),
                Expr::S1(x) => format!("S{}", x.arg()),
                Expr::S2(x, y) => format!("S{}{}", x.arg(), y.arg()),
                Expr::K0 => "K".to_string(),
                Expr::K1(x) => format!("K{}", x.arg()),
                Expr::W0 => "W".to_string(),
                Expr::W1(x) => format!("W{}", x.arg()),
                Expr::C0 => "C".to_string(),
                Expr::C1(x) => format!("C{}", x.arg()),
                Expr::C2(x, y) => format!("C{}{}", x.arg(), y.arg()),
                Expr::B0 => "B".to_string(),
                Expr::B1(x) => format!("B{}", x.arg()),
                Expr::B2(x, y) => format!("B{}{}", x.arg(), y.arg()),
            }
        )
    }
}

#[cfg(test)]
mod parse_tests {
    use std::collections::HashMap;

    use super::Expr;

    static SK_DEFS: HashMap<char, Expr> = HashMap::from([('S', Expr::S0), ('K', Expr::K0)]);

    #[test]
    fn S_Combinator() {
        let to_parse = &"S".to_string();
        let result = "S".to_string();
        let parsed = Expr::parse(to_parse, &Expr::SK_DEFS(), false).unwrap();
        assert_eq!(parsed.to_string(), result)
    }

    #[test]
    fn S_Combinator_eval() {
        let to_parse = &"Sxyz".to_string();
        let result = "xz(yz)".to_string();
        let parsed = Expr::parse(to_parse, &Expr::SK_DEFS(), false).unwrap();
        assert_eq!(parsed.to_string(), result)
    }

    #[test]
    fn K_Combinator() {
        let to_parse = &"K".to_string();
        let result = "K".to_string();
        let parsed = Expr::parse(to_parse, &Expr::SK_DEFS(), false).unwrap();
        assert_eq!(parsed.to_string(), result)
    }

    #[test]
    fn K_Combinator_eval() {
        let to_parse = &"Kxy".to_string();
        let result = "x".to_string();
        let parsed = Expr::parse(to_parse, &Expr::SK_DEFS(), false).unwrap();
        assert_eq!(parsed.to_string(), result)
    }
}

pub fn assignment(line: &str, d: &Defs, t: bool) -> Option<(char, Expr)> {
    let mut it = line.char_indices();
    let mut name = None;
    for (_, c) in it.by_ref() {
        if c != ' ' {
            name = Some(c);
            break;
        } else {
            continue;
        }
    }
    for (_, e) in it.by_ref() {
        if e == '=' {
            break;
        } else if e != ' ' {
            return None;
        }
    }
    Some((
        name?,
        Expr::parse(it.fold(String::new(), |a, b| a + &b.1.to_string()), d, t)?,
    ))
}

impl<'a> Expr {
    pub fn parse_file(body: &str, d: &mut Defs, t: bool) -> Option<Expr> {
        for line in body.lines() {
            let haystack = line.trim();
            if ["#", "//", "--"]
                .iter()
                .any(|s| s[..] == haystack[0..s.len()])
            {
                continue;
            } else if let Some((k, v)) = assignment(haystack, d, t) {
                d.insert(k, v);
            } else if let Some(e) = Expr::parse(haystack.to_string(), d, t) {
                return Some(e);
            }
        }
        None
    }

    pub fn apply(&'a self, other: Rc<Expr>, trace: bool) -> Rc<Expr> {
        let result = match self {
            Expr::LongVar(name) => Rc::new(Expr::LongVar(name.to_string() + &other.arg())),
            Expr::Variable(name) => Rc::new(Expr::LongVar(name.to_string() + &other.arg())),
            Expr::S0 => Rc::new(Expr::S1(other.clone())),
            Expr::S1(x) => Rc::new(Expr::S2(x.clone(), other.clone())),
            Expr::S2(x, y) => x
                .apply(other.clone(), trace)
                .apply(y.apply(other.clone(), trace), trace),
            Expr::K0 => Rc::new(Expr::K1(other.clone())),
            Expr::K1(x) => x.clone(),
            Expr::W0 => Rc::new(Expr::W1(other.clone())),
            Expr::W1(x) => x.apply(other.clone(), trace).apply(other.clone(), trace),
            Expr::C0 => Rc::new(Expr::C1(other.clone())),
            Expr::C1(x) => Rc::new(Expr::C2(x.clone(), other.clone())),
            Expr::C2(x, y) => x.apply(other.clone(), trace).apply(y.clone(), trace),
            Expr::B0 => Rc::new(Expr::B1(other.clone())),
            Expr::B1(x) => Rc::new(Expr::B2(x.clone(), other.clone())),
            Expr::B2(x, y) => x.apply(y.apply(other.clone(), trace), trace),
        };
        if trace {
            println!(
                "`{}` applied to `{}`, resulting in `{}`",
                self, other, result
            );
        }
        result
    }
    pub fn arg(&self) -> String {
        match self {
            Expr::Variable(name) => name.to_string(),
            Expr::LongVar(name) => format!("({})", name),
            Expr::S0 => String::from("S"),
            Expr::S1(x) => format!("(S{})", x.arg()),
            Expr::S2(x, y) => format!("(S{}{})", x.arg(), y.arg()),
            Expr::K0 => String::from("K"),
            Expr::K1(x) => format!("(K{})", x.arg()),
            Expr::W0 => String::from("W"),
            Expr::W1(x) => format!("(W{})", x.arg()),
            Expr::C0 => String::from("C"),
            Expr::C1(x) => format!("(C{})", x.arg()),
            Expr::C2(x, y) => format!("(C{}{})", x.arg(), y.arg()),
            Expr::B0 => String::from("B"),
            Expr::B1(x) => format!("(B{})", x.arg()),
            Expr::B2(x, y) => format!("(B{}{})", x.arg(), y.arg()),
        }
    }
    pub fn parse(s: String, d: &Defs, trace: bool) -> Option<Expr> {
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
                tokens.push_back(Expr::parse(running, d, trace)?);
            } else if d.contains_key(&c) {
                tokens.push_back(d.get(&c)?.clone());
            } else {
                tokens.push_back(Expr::Variable(c));
            }
        }
        if tokens.is_empty() {
            return None;
        }
        let mut out = tokens.pop_front()?;
        while !tokens.is_empty() {
            out = Expr::clone(&out.apply(Rc::new(tokens.pop_front()?), trace));
        }
        Some(out)
    }
}
