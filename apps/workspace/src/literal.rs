//! Spec modules as raea writes them: a value as a TypeScript literal, and whole numbers kept whole.
/// A value as a TypeScript literal for a spec module: unquoted keys where they can be, nulls and empty lists left out.
pub fn ts_literal(v: &serde_json::Value, indent: usize) -> String {
    use serde_json::Value;
    let (pad, end) = ("  ".repeat(indent + 1), "  ".repeat(indent));
    match v {
        Value::Array(a) if a.iter().all(Value::is_string) && v.to_string().len() < 70 => format!("[{}]", a.iter().map(Value::to_string).collect::<Vec<_>>().join(", ")),
        Value::Array(a) => format!("[\n{}{end}]", a.iter().map(|x| format!("{pad}{},\n", ts_literal(x, indent + 1))).collect::<String>()),
        Value::Object(o) => {
            let ident = |k: &str| k.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
            let kept = o.iter().filter(|(_, x)| !x.is_null() && !x.as_array().is_some_and(Vec::is_empty));
            let body: String = kept.map(|(k, x)| format!("{pad}{}: {},\n", if ident(k) { k.clone() } else { Value::String(k.clone()).to_string() }, ts_literal(x, indent + 1))).collect();
            format!("{{\n{body}{end}}}")
        }
        v => tidy(v).to_string(),
    }
}

/// A number that's whole (as f64 values from Rust are, 45.0) written as an integer.
pub fn tidy(v: &serde_json::Value) -> serde_json::Value {
    match v.as_f64() {
        Some(n) if v.is_f64() && n.fract() == 0.0 && n.abs() < 1e15 => serde_json::Value::from(n as i64),
        _ => match v {
            serde_json::Value::Array(a) => serde_json::Value::Array(a.iter().map(tidy).collect()),
            v => v.clone(),
        },
    }
}

/// `tidy` through a whole value: objects and arrays too.
pub fn tidy_deep(v: &serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(o) => serde_json::Value::Object(o.iter().filter(|(_, x)| !x.is_null()).map(|(k, x)| (k.clone(), tidy_deep(x))).collect()),
        serde_json::Value::Array(a) => serde_json::Value::Array(a.iter().map(tidy_deep).collect()),
        v => tidy(v),
    }
}
