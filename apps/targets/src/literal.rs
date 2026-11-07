//! A JavaScript literal read without running it (T0a's `export const app`): objects with bare or quoted keys, arrays,
//! strings in single or double quotes, numbers, true, false and null, trailing commas allowed. Anything else (a name,
//! a call, a spread, a template string, a computed key) is an error at its line.

use serde_json::{Map, Number, Value};

/// A refusal at a line of the file.
#[derive(Debug, PartialEq)]
pub struct Error {
    pub line: usize,
    pub why: String,
}

impl Error {
    /// `why` at byte `at` of `src`.
    pub fn at(src: &str, at: usize, why: &str) -> Error {
        Error { line: src[..at.min(src.len())].matches('\n').count() + 1, why: why.into() }
    }
}

/// `src` with each comment replaced by spaces (newlines kept), so byte offsets and lines still match `src`. Strings
/// are left alone, so a "//" inside one isn't a comment.
pub fn blank_comments(src: &str) -> String {
    let b = src.as_bytes();
    let mut out = b.to_vec();
    let (mut i, mut quote) = (0, None::<u8>);
    while i < b.len() {
        match quote {
            Some(q) => {
                if b[i] == b'\\' {
                    i += 1;
                } else if b[i] == q {
                    quote = None;
                }
            }
            None if b[i] == b'"' || b[i] == b'\'' || b[i] == b'`' => quote = Some(b[i]),
            None if b[i..].starts_with(b"//") => {
                while i < b.len() && b[i] != b'\n' {
                    out[i] = b' ';
                    i += 1;
                }
                continue;
            }
            None if b[i..].starts_with(b"/*") => {
                let end = src[i + 2..].find("*/").map_or(b.len(), |e| i + 2 + e + 2);
                for o in &mut out[i..end] {
                    if *o != b'\n' {
                        *o = b' ';
                    }
                }
                i = end;
                continue;
            }
            None => {}
        }
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| src.to_string())
}

/// `code` (comments already blanked) with every string's contents replaced by spaces, quotes kept: where to look for
/// an `export`, so one written inside a string isn't found.
pub fn blank_strings(code: &str) -> String {
    let b = code.as_bytes();
    let mut out = b.to_vec();
    let (mut i, mut quote) = (0, None::<u8>);
    while i < b.len() {
        match quote {
            Some(_) if b[i] == b'\\' => {
                out[i] = b' ';
                if i + 1 < b.len() && b[i + 1] != b'\n' {
                    out[i + 1] = b' ';
                }
                i += 1;
            }
            Some(q) if b[i] == q => quote = None,
            Some(_) if b[i] != b'\n' => out[i] = b' ',
            Some(_) => {}
            None if matches!(b[i], b'"' | b'\'' | b'`') => quote = Some(b[i]),
            None => {}
        }
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| code.to_string())
}

/// The literal starting at byte `at` of `code` (comments blanked), and the byte after it. `src` gives error lines.
pub fn value(src: &str, code: &str, at: usize) -> Result<(Value, usize), Error> {
    let b = code.as_bytes();
    let i = skip(b, at);
    let err = |at: usize, why: &str| Err(Error::at(src, at, why));
    match b.get(i) {
        Some(b'{') => object(src, code, i),
        Some(b'[') => array(src, code, i),
        Some(b'"' | b'\'') => string(src, code, i).map(|(s, e)| (Value::String(s), e)),
        Some(b'`') => err(i, "a template string isn't a literal the CLI can read; use quotes"),
        Some(c) if c.is_ascii_digit() || *c == b'-' || *c == b'.' => number(src, code, i),
        Some(_) => {
            let word: String = code[i..].chars().take_while(|ch| ch.is_alphanumeric() || *ch == '_' || *ch == '$').collect();
            match word.as_str() {
                "true" => Ok((Value::Bool(true), i + 4)),
                "false" => Ok((Value::Bool(false), i + 5)),
                "null" => Ok((Value::Null, i + 4)),
                "" => err(i, "expected a literal here"),
                w => err(i, &format!("`{w}` isn't a literal; the CLI reads `app` without running the file")),
            }
        }
        None => err(i, "the file ends inside `app`"),
    }
}

fn skip(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

fn object(src: &str, code: &str, open: usize) -> Result<(Value, usize), Error> {
    let b = code.as_bytes();
    let mut m = Map::new();
    let mut i = skip(b, open + 1);
    loop {
        match b.get(i) {
            Some(b'}') => return Ok((Value::Object(m), i + 1)),
            None => return Err(Error::at(src, open, "`{` is never closed")),
            Some(b'.') => return Err(Error::at(src, i, "a spread isn't a literal the CLI can read")),
            Some(b'[') => return Err(Error::at(src, i, "a computed key isn't a literal the CLI can read")),
            _ => {}
        }
        let (key, after) = if matches!(b[i], b'"' | b'\'') {
            string(src, code, i)?
        } else {
            let k: String = code[i..].chars().take_while(|ch| ch.is_alphanumeric() || *ch == '_' || *ch == '$').collect();
            if k.is_empty() {
                return Err(Error::at(src, i, "expected a key"));
            }
            let n = i + k.len();
            (k, n)
        };
        i = skip(b, after);
        if b.get(i) != Some(&b':') {
            return Err(Error::at(src, i, &format!("`{key}` needs `: value` (shorthand names a variable, which the CLI can't read)")));
        }
        let (v, e) = value(src, code, i + 1)?;
        m.insert(key, v);
        i = skip(b, e);
        match b.get(i) {
            Some(b',') => i = skip(b, i + 1),
            Some(b'}') => {}
            _ => return Err(Error::at(src, i, "expected `,` or `}`")),
        }
    }
}

fn array(src: &str, code: &str, open: usize) -> Result<(Value, usize), Error> {
    let b = code.as_bytes();
    let mut v = Vec::new();
    let mut i = skip(b, open + 1);
    loop {
        match b.get(i) {
            Some(b']') => return Ok((Value::Array(v), i + 1)),
            None => return Err(Error::at(src, open, "`[` is never closed")),
            Some(b'.') if b[i..].starts_with(b"...") => return Err(Error::at(src, i, "a spread isn't a literal the CLI can read")),
            _ => {}
        }
        let (x, e) = value(src, code, i)?;
        v.push(x);
        i = skip(b, e);
        match b.get(i) {
            Some(b',') => i = skip(b, i + 1),
            Some(b']') => {}
            _ => return Err(Error::at(src, i, "expected `,` or `]`")),
        }
    }
}

fn string(src: &str, code: &str, open: usize) -> Result<(String, usize), Error> {
    let b = code.as_bytes();
    let q = b[open];
    let mut s = String::new();
    let mut i = open + 1;
    while i < b.len() {
        match b[i] {
            c if c == q => return Ok((s, i + 1)),
            b'\\' if i + 1 < b.len() => {
                let n = b[i + 1] as char;
                s.push(match n {
                    'n' => '\n',
                    't' => '\t',
                    other => other,
                });
                i += 2;
            }
            b'\n' => break,
            _ => {
                let ch = code[i..].chars().next().unwrap();
                s.push(ch);
                i += ch.len_utf8();
            }
        }
    }
    Err(Error::at(src, open, "a string is never closed"))
}

fn number(src: &str, code: &str, at: usize) -> Result<(Value, usize), Error> {
    let t: String = code[at..].chars().take_while(|ch| ch.is_ascii_digit() || matches!(ch, '-' | '.' | 'e' | 'E' | '+' | '_')).collect();
    let n: f64 = t.replace('_', "").parse().map_err(|_| Error::at(src, at, &format!("`{t}` isn't a number")))?;
    let v = Number::from_f64(n).ok_or_else(|| Error::at(src, at, "a number that isn't finite"))?;
    Ok((Value::Number(v), at + t.len()))
}
