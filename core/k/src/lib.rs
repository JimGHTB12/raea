//! K on the GPU. A kernel is a few lines of a small K dialect; it runs once per element `i` of an output, every
//! intermediate fused into one WGSL compute shader (no temporaries in memory between verbs). The same text runs on
//! the CPU through an interpreter, and the two must agree bit for bit: everything is i32 with wrapping arithmetic,
//! truncating division, and fixed loop orders, so there is nothing for a GPU to reorder.
//!
//! The dialect (K's right-to-left evaluation, no precedence: `a*b+c` is `a*(b+c)`):
//!   name:expr          a binding (later lines see it); an output is a binding the caller names as one
//!   + - * %            add, subtract, multiply, divide (truncating); `-x` at the start of an expression negates
//!   !                  mod (truncating remainder)
//!   & |                min, max (K's)
//!   < > =              comparisons, 1 or 0
//!   i                  the element's index
//!   byte[b;k] word[b;k]          the k-th byte / u32 word of input buffer b
//!   pop[x] xor[a;b] or[a;b] and[a;b] shl[a;n] shr[a;n]   bits (shr is arithmetic)
//!   clamp[x;lo;hi] sel[c;a;b] pack[r;g;b;a]               clamp, choose, four bytes into a word
//!   sum[k;n;expr]      +/ over k = 0..n-1 (K's over, in a fixed order)
//!   low[k;n;expr]      &/ over k = 0..n-1: the least (i32 max over none)
//! Lines starting with `/` are comments, as in K.

mod native;

pub use native::*;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum E {
    Num(i64),
    Name(String),
    Call(String, Vec<E>),
    Bin(char, Box<E>, Box<E>),
    Neg(Box<E>),
}

#[derive(Debug, Clone)]
pub struct Kernel {
    pub lines: Vec<Line>,
}

/// A kernel line: a binding, or a guarded block whose lines run only where its condition is non-zero (names bound
/// outside keep their values where it isn't).
#[derive(Debug, Clone)]
pub enum Line {
    Bind(String, E),
    When(E, Vec<Line>),
}

// === parsing ===

#[derive(Debug, Clone, PartialEq)]
enum T {
    Num(i64),
    Name(String),
    Verb(char),
    Open,
    Close,
    LBr,
    RBr,
    Semi,
    Colon,
}

fn lex(s: &str) -> Result<Vec<T>, String> {
    let c: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < c.len() {
        let ch = c[i];
        if ch.is_whitespace() {
            i += 1;
        } else if ch.is_ascii_digit() {
            let st = i;
            while i < c.len() && c[i].is_ascii_digit() {
                i += 1;
            }
            out.push(T::Num(c[st..i].iter().collect::<String>().parse().map_err(|e| format!("{e}"))?));
        } else if ch.is_alphabetic() || ch == '_' {
            let st = i;
            while i < c.len() && (c[i].is_alphanumeric() || c[i] == '_') {
                i += 1;
            }
            out.push(T::Name(c[st..i].iter().collect()));
        } else {
            out.push(match ch {
                '(' => T::Open,
                ')' => T::Close,
                '[' => T::LBr,
                ']' => T::RBr,
                ';' => T::Semi,
                ':' => T::Colon,
                '+' | '-' | '*' | '%' | '!' | '&' | '|' | '<' | '>' | '=' => T::Verb(ch),
                other => return Err(format!("unexpected '{other}'")),
            });
            i += 1;
        }
    }
    Ok(out)
}

struct P {
    t: Vec<T>,
    i: usize,
}

impl P {
    fn peek(&self) -> Option<&T> {
        self.t.get(self.i)
    }
    fn next(&mut self) -> Option<T> {
        self.i += 1;
        self.t.get(self.i - 1).cloned()
    }
    fn expect(&mut self, t: T) -> Result<(), String> {
        match self.next() {
            Some(x) if x == t => Ok(()),
            other => Err(format!("expected {t:?}, got {other:?}")),
        }
    }
    /// expr := '-' expr | noun (verb expr)?      (right to left: the verb takes everything to its right)
    fn expr(&mut self) -> Result<E, String> {
        if self.peek() == Some(&T::Verb('-')) {
            self.next();
            return Ok(E::Neg(Box::new(self.expr()?)));
        }
        let n = self.noun()?;
        if let Some(T::Verb(v)) = self.peek().cloned() {
            self.next();
            return Ok(E::Bin(v, Box::new(n), Box::new(self.expr()?)));
        }
        Ok(n)
    }
    fn noun(&mut self) -> Result<E, String> {
        match self.next() {
            Some(T::Num(n)) => Ok(E::Num(n)),
            Some(T::Open) => {
                let e = self.expr()?;
                self.expect(T::Close)?;
                Ok(e)
            }
            Some(T::Name(n)) => {
                if self.peek() == Some(&T::LBr) {
                    self.next();
                    let mut args = vec![self.expr()?];
                    while self.peek() == Some(&T::Semi) {
                        self.next();
                        args.push(self.expr()?);
                    }
                    self.expect(T::RBr)?;
                    Ok(E::Call(n, args))
                } else {
                    Ok(E::Name(n))
                }
            }
            other => Err(format!("expected a noun, got {other:?}")),
        }
    }
}

pub fn parse(src: &str) -> Result<Kernel, String> {
    // `when cond` opens a block, `end` closes it
    let mut stack: Vec<(Option<E>, Vec<Line>)> = vec![(None, Vec::new())];
    for raw in src.lines() {
        let l = raw.trim();
        if l.is_empty() || l.starts_with('/') {
            continue;
        }
        if l == "end" {
            let (cond, body) = stack.pop().ok_or("end without when")?;
            let cond = cond.ok_or("end without when")?;
            stack.last_mut().ok_or("end without when")?.1.push(Line::When(cond, body));
            continue;
        }
        if let Some(c) = l.strip_prefix("when ") {
            let mut p = P { t: lex(c)?, i: 0 };
            let e = p.expr().map_err(|e| format!("{l}: {e}"))?;
            stack.push((Some(e), Vec::new()));
            continue;
        }
        let mut p = P { t: lex(l)?, i: 0 };
        let name = match (p.t.first(), p.t.get(1)) {
            (Some(T::Name(n)), Some(T::Colon)) => {
                let n = n.clone();
                p.i = 2;
                n
            }
            _ => return Err(format!("every line binds a name (name:expr): {l}")),
        };
        let e = p.expr().map_err(|e| format!("{l}: {e}"))?;
        if p.i != p.t.len() {
            return Err(format!("{l}: trailing tokens"));
        }
        stack.last_mut().unwrap().1.push(Line::Bind(name, e));
    }
    if stack.len() != 1 {
        return Err("a when without its end".into());
    }
    Ok(Kernel { lines: stack.pop().unwrap().1 })
}

// === the CPU interpreter (the reference) ===

pub struct Cpu<'a> {
    pub buffers: HashMap<&'a str, &'a [u32]>,
    pub scalars: HashMap<&'a str, i32>,
}

impl Cpu<'_> {
    fn ev(&self, e: &E, env: &mut Vec<(String, i32)>) -> Result<i32, String> {
        Ok(match e {
            E::Num(n) => *n as i32,
            E::Name(n) => match env.iter().rev().find(|(k, _)| k == n) {
                Some((_, v)) => *v,
                None => *self.scalars.get(n.as_str()).ok_or(format!("unknown name {n}"))?,
            },
            E::Neg(a) => self.ev(a, env)?.wrapping_neg(),
            E::Bin(op, a, b) => {
                let (x, y) = (self.ev(a, env)?, self.ev(b, env)?);
                match op {
                    '+' => x.wrapping_add(y),
                    '-' => x.wrapping_sub(y),
                    '*' => x.wrapping_mul(y),
                    '%' => x.wrapping_div(y),
                    '!' => x.wrapping_rem(y),
                    '&' => x.min(y),
                    '|' => x.max(y),
                    '<' => (x < y) as i32,
                    '>' => (x > y) as i32,
                    _ => (x == y) as i32,
                }
            }
            E::Call(f, a) => {
                let buf = |k: usize| -> Result<&[u32], String> {
                    match &a[k] {
                        E::Name(n) => self.buffers.get(n.as_str()).copied().ok_or(format!("{f}: no buffer {n}")),
                        _ => Err(format!("{f}: the first argument is a buffer name")),
                    }
                };
                match f.as_str() {
                    // tex[T;x;y]: one RGBA texel as a packed word (the CPU holds T as packed words, T_w wide)
                    "tex" => {
                        let E::Name(t) = &a[0] else { return Err("tex[T;x;y]: T is a texture name".into()) };
                        let w = *self.scalars.get(format!("{t}_w").as_str()).ok_or(format!("tex: no scalar {t}_w"))?;
                        let (x, y) = (self.ev(&a[1], env)?, self.ev(&a[2], env)?);
                        buf(0)?[(y * w + x) as usize] as i32
                    }
                    "byte" => {
                        let k = self.ev(&a[1], env)? as u32;
                        ((buf(0)?[(k >> 2) as usize] >> (8 * (k & 3))) & 255) as i32
                    }
                    "word" => buf(0)?[self.ev(&a[1], env)? as usize] as i32,
                    "pop" => (self.ev(&a[0], env)? as u32).count_ones() as i32,
                    "xor" => self.ev(&a[0], env)? ^ self.ev(&a[1], env)?,
                    "or" => self.ev(&a[0], env)? | self.ev(&a[1], env)?,
                    "and" => self.ev(&a[0], env)? & self.ev(&a[1], env)?,
                    "shl" => self.ev(&a[0], env)?.wrapping_shl(self.ev(&a[1], env)? as u32),
                    "shr" => self.ev(&a[0], env)?.wrapping_shr(self.ev(&a[1], env)? as u32),
                    "clamp" => {
                        let (x, lo, hi) = (self.ev(&a[0], env)?, self.ev(&a[1], env)?, self.ev(&a[2], env)?);
                        x.max(lo).min(hi)
                    }
                    "sel" => {
                        if self.ev(&a[0], env)? != 0 {
                            self.ev(&a[1], env)?
                        } else {
                            self.ev(&a[2], env)?
                        }
                    }
                    "pack" => {
                        let b = |k: usize, env: &mut Vec<(String, i32)>| self.ev(&a[k], env).map(|v| (v as u32) & 255);
                        (b(0, env)? | (b(1, env)? << 8) | (b(2, env)? << 16) | (b(3, env)? << 24)) as i32
                    }
                    "sum" | "low" => {
                        let E::Name(k) = &a[0] else { return Err(format!("{f}[k;n;expr]: k is a name")) };
                        let n = self.ev(&a[1], env)?;
                        let low = f == "low";
                        let mut s = if low { i32::MAX } else { 0i32 };
                        for j in 0..n {
                            env.push((k.clone(), j));
                            let v = self.ev(&a[2], env)?;
                            s = if low { s.min(v) } else { s.wrapping_add(v) };
                            env.pop();
                        }
                        s
                    }
                    other => return Err(format!("unknown function {other}")),
                }
            }
        })
    }

    fn lines(&self, lines: &[Line], env: &mut Vec<(String, i32)>) -> Result<(), String> {
        for l in lines {
            match l {
                Line::Bind(name, e) => {
                    let v = self.ev(e, env)?;
                    env.push((name.clone(), v));
                }
                Line::When(c, body) => {
                    if self.ev(c, env)? != 0 {
                        self.lines(body, env)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Every output for elements 0..n.
    pub fn run(&self, k: &Kernel, n: usize, outputs: &[&str]) -> Result<Vec<Vec<u32>>, String> {
        let mut out = vec![Vec::with_capacity(n); outputs.len()];
        let mut env = Vec::new();
        for i in 0..n {
            env.clear();
            env.push(("i".to_string(), i as i32));
            self.lines(&k.lines, &mut env)?;
            for (o, name) in outputs.iter().enumerate() {
                out[o].push(env.iter().rev().find(|(k, _)| k == name).ok_or(format!("no output {name}"))?.1 as u32);
            }
        }
        Ok(out)
    }
}
