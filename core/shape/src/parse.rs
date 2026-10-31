//! A shape expression read into calls (`box(1, 2, 3)`, `minus(a, b)`) and their arguments.

use manifold_rust::manifold::Manifold;

use crate::build::call;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Tok {
    Name(String),
    Num(f64),
    Str(String),
    Open,
    Close,
    Comma,
}

pub(crate) fn lex(s: &str) -> Result<Vec<Tok>, String> {
    let c: Vec<char> = s.chars().collect();
    let (mut i, mut out) = (0, Vec::new());
    while i < c.len() {
        let ch = c[i];
        match ch {
            ' ' | '\t' | '\n' => i += 1,
            '(' => {
                out.push(Tok::Open);
                i += 1
            }
            ')' => {
                out.push(Tok::Close);
                i += 1
            }
            ',' => {
                out.push(Tok::Comma);
                i += 1
            }
            '"' | '\'' => {
                let end = c[i + 1..].iter().position(|x| *x == ch).ok_or("a string with no closing quote")? + i + 1;
                out.push(Tok::Str(c[i + 1..end].iter().collect()));
                i = end + 1;
            }
            d if d.is_ascii_digit() || d == '-' || d == '.' => {
                let start = i;
                i += 1;
                while i < c.len() && (c[i].is_ascii_digit() || c[i] == '.' || c[i] == 'e') {
                    i += 1;
                }
                let t: String = c[start..i].iter().collect();
                out.push(Tok::Num(t.parse().map_err(|_| format!("not a number: {t}"))?));
            }
            a if a.is_alphabetic() || a == '_' => {
                let start = i;
                while i < c.len() && (c[i].is_alphanumeric() || c[i] == '_' || c[i] == '3') {
                    i += 1;
                }
                out.push(Tok::Name(c[start..i].iter().collect()));
            }
            o => return Err(format!("unexpected {o:?} in a shape")),
        }
    }
    Ok(out)
}

#[derive(Clone)]
pub(crate) enum Arg {
    Num(f64),
    Str(String),
    Shape(Manifold),
}

pub(crate) struct P {
    pub(crate) t: Vec<Tok>,
    pub(crate) i: usize,
}

impl P {
    fn next(&mut self) -> Option<Tok> {
        let t = self.t.get(self.i).cloned();
        self.i += 1;
        t
    }

    pub(crate) fn expr(&mut self) -> Result<Arg, String> {
        match self.next() {
            Some(Tok::Num(n)) => Ok(Arg::Num(n)),
            Some(Tok::Str(s)) => Ok(Arg::Str(s)),
            Some(Tok::Name(f)) => {
                if self.next() != Some(Tok::Open) {
                    return Err(format!("{f} takes arguments: {f}(...)"));
                }
                let mut args = Vec::new();
                if self.t.get(self.i) == Some(&Tok::Close) {
                    self.i += 1;
                } else {
                    loop {
                        args.push(self.expr()?);
                        match self.next() {
                            Some(Tok::Comma) => continue,
                            Some(Tok::Close) => break,
                            _ => return Err(format!("{f}(...): a comma or a closing parenthesis")),
                        }
                    }
                }
                call(&f, args).map(Arg::Shape)
            }
            t => Err(format!("a shape, a number or a string, not {t:?}")),
        }
    }
}

pub(crate) fn nums(f: &str, a: &[Arg], n: usize) -> Result<Vec<f64>, String> {
    let v: Vec<f64> = a.iter().filter_map(|x| if let Arg::Num(n) = x { Some(*n) } else { None }).collect();
    if v.len() < n {
        return Err(format!("{f} needs {n} numbers"));
    }
    Ok(v)
}

pub(crate) fn shapes(f: &str, a: Vec<Arg>) -> Result<Vec<Manifold>, String> {
    let v: Vec<Manifold> = a.into_iter().filter_map(|x| if let Arg::Shape(m) = x { Some(m) } else { None }).collect();
    if v.is_empty() {
        return Err(format!("{f} needs shapes"));
    }
    Ok(v)
}
