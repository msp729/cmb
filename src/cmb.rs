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
    I,
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "{}",
            match self {
                Self::Variable(c) => c.to_string(),
                Self::LongVar(name) => name.clone(),
                Self::S0 => "S".to_string(),
                Self::S1(x) => format!("S{}", x.arg()),
                Self::S2(x, y) => format!("S{}{}", x.arg(), y.arg()),
                Self::K0 => "K".to_string(),
                Self::K1(x) => format!("K{}", x.arg()),
                Self::W0 => "W".to_string(),
                Self::W1(x) => format!("W{}", x.arg()),
                Self::C0 => "C".to_string(),
                Self::C1(x) => format!("C{}", x.arg()),
                Self::C2(x, y) => format!("C{}{}", x.arg(), y.arg()),
                Self::B0 => "B".to_string(),
                Self::B1(x) => format!("B{}", x.arg()),
                Self::B2(x, y) => format!("B{}{}", x.arg(), y.arg()),
                Self::I => "I".to_string(),
            }
        )
    }
}

pub fn assignment(line: &str, d: &Defs, t: bool) -> Option<(char, Expr)> {
    if t {
        println!("Checking for assignment in `{line}`");
    }
    let mut it = line.char_indices();
    let mut name = None;
    for (_, c) in it.by_ref() {
        if c != ' ' {
            name = Some(c);
            break;
        }
    }
    if t {
        println!("name: {}", name?);
    }
    for (_, e) in it.by_ref() {
        if e == '=' {
            if t {
                println!("e: {e}");
            }
            break;
        }
        if e != ' ' {
            return None;
        }
    }
    Some((
        name?,
        Expr::parse(&it.fold(String::new(), |a, b| a + &b.1.to_string()), d, t)?,
    ))
}

impl<'a> Expr {
    pub fn parse_file(body: &str, d: &mut Defs, t: bool) -> Option<Self> {
        for line in body.lines() {
            let haystack = line.trim();
            if ["#", "//", "--"]
                .iter()
                .any(|s| s[..] == haystack[0..s.len()])
            {
                continue;
            }
            if let Some((k, v)) = assignment(haystack, d, t) {
                d.insert(k, v);
            } else if let Some(e) = Self::parse(haystack, d, t) {
                return Some(e);
            }
        }
        None
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn apply(&'a self, other: Rc<Self>, trace: bool) -> Rc<Self> {
        let result = match self {
            Self::LongVar(name) => Rc::new(Self::LongVar(name.to_string() + &other.arg())),
            Self::Variable(name) => Rc::new(Self::LongVar(name.to_string() + &other.arg())),
            Self::S0 => Rc::new(Self::S1(other.clone())),
            Self::S1(x) => Rc::new(Self::S2(x.clone(), other.clone())),
            Self::S2(x, y) => x
                .apply(other.clone(), trace)
                .apply(y.apply(other.clone(), trace), trace),
            Self::K0 => Rc::new(Self::K1(other.clone())),
            Self::K1(x) => x.clone(),
            Self::W0 => Rc::new(Self::W1(other.clone())),
            Self::W1(x) => x.apply(other.clone(), trace).apply(other.clone(), trace),
            Self::C0 => Rc::new(Self::C1(other.clone())),
            Self::C1(x) => Rc::new(Self::C2(x.clone(), other.clone())),
            Self::C2(x, y) => x.apply(other.clone(), trace).apply(y.clone(), trace),
            Self::B0 => Rc::new(Self::B1(other.clone())),
            Self::B1(x) => Rc::new(Self::B2(x.clone(), other.clone())),
            Self::B2(x, y) => x.apply(y.apply(other.clone(), trace), trace),
            Self::I => other.clone(),
        };
        if trace {
            println!("`{self}` applied to `{other}`, resulting in `{result}`");
        }
        result
    }

    pub fn arg(&self) -> String {
        match self {
            Self::Variable(name) => name.to_string(),
            Self::LongVar(name) => format!("({name})"),
            Self::S0 => String::from("S"),
            Self::S1(x) => format!("(S{})", x.arg()),
            Self::S2(x, y) => format!("(S{}{})", x.arg(), y.arg()),
            Self::K0 => String::from("K"),
            Self::K1(x) => format!("(K{})", x.arg()),
            Self::W0 => String::from("W"),
            Self::W1(x) => format!("(W{})", x.arg()),
            Self::C0 => String::from("C"),
            Self::C1(x) => format!("(C{})", x.arg()),
            Self::C2(x, y) => format!("(C{}{})", x.arg(), y.arg()),
            Self::B0 => String::from("B"),
            Self::B1(x) => format!("(B{})", x.arg()),
            Self::B2(x, y) => format!("(B{}{})", x.arg(), y.arg()),
            Self::I => "I".to_string(),
        }
    }

    pub fn parse(s: &str, d: &Defs, trace: bool) -> Option<Self> {
        let mut tokens: VecDeque<Self> = VecDeque::new();
        let mut to_ignore = 0;
        for (i, c) in s.chars().enumerate() {
            if to_ignore != 0 {
                to_ignore -= 1;
                continue;
            }
            if c == ' ' {
                continue;
            }
            if c == '(' {
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
                tokens.push_back(Self::parse(&running, d, trace)?);
            } else if d.contains_key(&c) {
                tokens.push_back(d.get(&c)?.clone());
            } else {
                tokens.push_back(Self::Variable(c));
            }
        }
        if tokens.is_empty() {
            return None;
        }
        let mut out = tokens.pop_front()?;
        while !tokens.is_empty() {
            out = Self::clone(&out.apply(Rc::new(tokens.pop_front()?), trace));
        }
        Some(out)
    }
}
