//! A picked element's prop changed on the play stage, written into the game's composition as its next version
//! (docs/handoff/feed.md, tools on play items, step 4). The stage names an element by the component it was rendered
//! in and its kind (live play's `data-raea-owner`), so the file's element is found as the app's host finds a clip's
//! <Audio> (apps/app/host/clips.ts): the elements of that kind inside that component that set the prop. The prop's
//! value is written where it's written out: the literal itself (`height: 300`), or the top-level constant it names
//! (`height: PADDLE_LEN` changes `const PADDLE_LEN = 300`, as the app's retune does: host/edit.ts withTune). A value
//! worked out in the file (`x - PADDLE_TH / 2`) is refused: change it there.

use std::path::{Path, PathBuf};

use oxc::allocator::Allocator;
use oxc::ast::ast::*;
use oxc::ast_visit::{Visit, walk};
use oxc::parser::Parser;
use oxc::span::{GetSpan, SourceType, Span};
use serde_json::Value;

/// One change: the element (its component's name and its kind), the prop (`style.height`, or a prop's own name) and
/// its new value.
pub struct Tweak<'s> {
    pub owner: &'s str,
    pub kind: &'s str,
    pub prop: &'s str,
    pub value: &'s Value,
}

/// Where a prop's value is written: its literal, or the constant it names.
enum At {
    Literal(Span),
    Constant(String, Span),
}

/// What an element gives a prop, as written: a literal, a name, or something worked out.
enum Found {
    Literal(Span),
    Name(String),
    Worked(Span),
}

/// The JSX elements of a kind (inside `within`) and what each gives the prop.
struct Uses<'t> {
    kind: &'t str,
    prop: &'t str,
    within: Span,
    found: Vec<Found>,
}

impl<'a> Visit<'a> for Uses<'_> {
    fn visit_jsx_opening_element(&mut self, it: &JSXOpeningElement<'a>) {
        let inside = self.within.start <= it.span.start && it.span.end <= self.within.end;
        if inside && matches!(&it.name, JSXElementName::Identifier(n) if n.name == self.kind) {
            match prop_of(it, self.prop) {
                Some(Given::Text(s)) => self.found.push(Found::Literal(s)),
                Some(Given::Expr(e)) => self.found.push(match e.without_parentheses() {
                    e if literal(e) => Found::Literal(e.span()),
                    Expression::Identifier(i) => Found::Name(i.name.to_string()),
                    e => Found::Worked(e.span()),
                }),
                None => {}
            }
        }
        walk::walk_jsx_opening_element(self, it);
    }
}

/// A prop as an element gives it: `k="text"`, or an expression.
enum Given<'b, 'a> {
    Text(Span),
    Expr(&'b Expression<'a>),
}

/// What an element gives `prop`: `style.KEY` is a key of its `style={{ ... }}`, anything else an attribute.
fn prop_of<'b, 'a>(el: &'b JSXOpeningElement<'a>, prop: &str) -> Option<Given<'b, 'a>> {
    let (attr, key) = prop.split_once('.').map_or((prop, None), |(a, k)| (a, Some(k)));
    let value = el.attributes.iter().find_map(|a| match a {
        JSXAttributeItem::Attribute(a) if matches!(&a.name, JSXAttributeName::Identifier(n) if n.name == attr) => a.value.as_ref(),
        _ => None,
    })?;
    let e = match value {
        JSXAttributeValue::StringLiteral(s) if key.is_none() => return Some(Given::Text(s.span)),
        JSXAttributeValue::ExpressionContainer(c) => c.expression.as_expression()?,
        _ => return None,
    };
    let Some(key) = key else { return Some(Given::Expr(e)) };
    let Expression::ObjectExpression(o) = e.without_parentheses() else { return None };
    o.properties.iter().find_map(|p| match p {
        ObjectPropertyKind::ObjectProperty(p) if p.key.static_name().is_some_and(|n| n == key) => Some(Given::Expr(&p.value)),
        _ => None,
    })
}

/// The component named `owner` at the file's top level: a function declaration, or a const holding an arrow or a
/// function (exported or not).
fn component<'b, 'a>(program: &'b Program<'a>, owner: &str) -> Option<Span> {
    for st in &program.body {
        let decl = match st {
            Statement::ExportDeclaration(e) => Some(&e.declaration),
            Statement::ExportDefaultDeclaration(e) => match &e.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(f) if f.id.as_ref().is_some_and(|i| i.name == owner) => return Some(f.span),
                _ => None,
            },
            st => st.as_declaration(),
        };
        match decl {
            Some(Declaration::FunctionDeclaration(f)) if f.id.as_ref().is_some_and(|i| i.name == owner) => return Some(f.span),
            Some(Declaration::VariableDeclaration(v)) => {
                for d in &v.declarations {
                    if matches!(&d.id, BindingPattern::BindingIdentifier(i) if i.name == owner)
                        && let Some(init) = &d.init
                        && matches!(init.without_parentheses(), Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_))
                    {
                        return Some(init.span());
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// The file's top-level `const NAME = <number or string>` literals, by name.
fn constants<'a>(program: &Program<'a>) -> Vec<(String, Span)> {
    let mut out = vec![];
    for st in &program.body {
        let decl = match st {
            Statement::ExportDeclaration(e) => Some(&e.declaration),
            st => st.as_declaration(),
        };
        let Some(Declaration::VariableDeclaration(v)) = decl else { continue };
        if v.kind != VariableDeclarationKind::Const {
            continue;
        }
        for d in &v.declarations {
            if let (BindingPattern::BindingIdentifier(i), Some(init)) = (&d.id, &d.init)
                && literal(init)
            {
                out.push((i.name.to_string(), init.span()));
            }
        }
    }
    out
}

/// A number or a string written out (`-4` too).
fn literal(e: &Expression) -> bool {
    match e.without_parentheses() {
        Expression::NumericLiteral(_) | Expression::StringLiteral(_) => true,
        Expression::UnaryExpression(u) => u.operator == UnaryOperator::UnaryNegation && matches!(u.argument, Expression::NumericLiteral(_)),
        Expression::TemplateLiteral(t) => t.expressions.is_empty(),
        _ => false,
    }
}

/// The value as the file writes it.
fn written(v: &Value) -> Result<String, String> {
    match v {
        Value::Number(n) => Ok(n.to_string()),
        Value::String(s) => Ok(serde_json::to_string(s).unwrap()),
        Value::Bool(b) => Ok(b.to_string()),
        _ => Err("a prop is set to a number, a string or true/false".into()),
    }
}

/// The source with the tweaks written in; refused when an element or its value can't be found as written out.
pub fn tweak(src: &str, tweaks: &[Tweak]) -> Result<String, String> {
    let alloc = Allocator::default();
    let ret = Parser::new(&alloc, src, SourceType::tsx()).parse();
    if let Some(e) = ret.diagnostics.errors().next() {
        return Err(format!("the composition doesn't parse: {e}"));
    }
    let program = ret.program;
    let consts = constants(&program);
    let mut splices: Vec<(Span, String)> = vec![];
    for t in tweaks {
        let what = format!("{} in {}", t.kind, t.owner);
        let span = component(&program, t.owner).ok_or_else(|| format!("no component named {} at the top of the file", t.owner))?;
        let mut uses = Uses { kind: t.kind, prop: t.prop, within: span, found: vec![] };
        uses.visit_program(&program);
        let mut ats: Vec<At> = vec![];
        for f in uses.found {
            ats.push(match f {
                Found::Literal(s) => At::Literal(s),
                Found::Name(n) => match consts.iter().find(|(c, _)| *c == n) {
                    Some((c, s)) => At::Constant(c.clone(), *s),
                    None => return Err(format!("{what}'s {} is {n}, not a constant written out at the top of the file: change it there", t.prop)),
                },
                Found::Worked(s) => return Err(format!("{what}'s {} is worked out in the file ({}): change it there", t.prop, &src[s.start as usize..s.end as usize])),
            });
        }
        let at = match ats.as_slice() {
            [] => return Err(format!("no {what} sets {} in the file", t.prop)),
            [one] => one,
            // several elements, one constant: the change is the constant's
            [first @ At::Constant(n, _), rest @ ..] if rest.iter().all(|a| matches!(a, At::Constant(m, _) if m == n)) => first,
            _ => return Err(format!("{} {what} elements set {}: give the one to change a key or a component of its own", ats.len(), t.prop)),
        };
        let span = match at {
            At::Literal(s) | At::Constant(_, s) => *s,
        };
        splices.push((span, written(t.value)?));
    }
    splices.sort_by_key(|(s, _)| std::cmp::Reverse(s.start));
    splices.dedup_by_key(|(s, _)| s.start);
    let mut out = src.to_string();
    for (s, text) in splices {
        out.replace_range(s.start as usize..s.end as usize, &text);
    }
    Ok(out)
}

/// The next version's name beside `tsx` that isn't taken: NAME.tsx -> NAME-v2.tsx, NAME-v2.tsx -> NAME-v3.tsx, ...
/// (the app's host/edit.ts freeName).
pub fn free_name(tsx: &Path) -> PathBuf {
    let next = |p: &Path| {
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("composition");
        let (base, n) = match stem.rsplit_once("-v").and_then(|(b, n)| n.parse::<u32>().ok().map(|n| (b, n))) {
            Some((b, n)) => (b.to_string(), n + 1),
            None => (stem.to_string(), 2),
        };
        p.with_file_name(format!("{base}-v{n}.tsx"))
    };
    let mut p = next(tsx);
    while p.exists() {
        p = next(&p);
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const GAME: &str = r#"import { AbsoluteFill } from "raea";
export const PADDLE_LEN = 300;
const EDGE = 8;
function Paddle({ x, y }: { x: number; y: number }) {
  return (
    <div style={{ position: "absolute", left: x - 28, height: PADDLE_LEN, background: "red", borderRadius: 12 }}>
      <div style={{ left: EDGE, width: 9 }} />
    </div>
  );
}
const Ball = () => <div style={{ width: 32, height: 32 }} title="ball" />;
export function Game() {
  return <AbsoluteFill><Paddle x={1} y={2} /><Paddle x={3} y={4} /><Ball /></AbsoluteFill>;
}
"#;
    fn one(owner: &str, kind: &str, prop: &str, v: Value) -> Result<String, String> {
        tweak(GAME, &[Tweak { owner, kind, prop, value: &v }])
    }

    #[test]
    fn a_constant_it_names_is_changed() {
        let out = one("Paddle", "div", "style.height", json!(400)).unwrap();
        assert!(out.contains("export const PADDLE_LEN = 400;"));
        assert!(out.contains("height: PADDLE_LEN"));
    }

    #[test]
    fn a_literal_is_changed_in_place() {
        assert!(one("Paddle", "div", "style.borderRadius", json!(20)).unwrap().contains("borderRadius: 20 "));
        assert!(one("Paddle", "div", "style.background", json!("blue")).unwrap().contains(r#"background: "blue""#));
        assert!(one("Ball", "div", "title", json!("puck")).unwrap().contains(r#"title="puck""#));
        assert!(one("Ball", "div", "style.width", json!(40)).unwrap().contains("width: 40, height: 32"));
    }

    #[test]
    fn worked_out_or_missing_is_refused() {
        assert!(one("Paddle", "div", "style.left", json!(5)).unwrap_err().contains("worked out"));
        assert!(one("Paddle", "span", "style.left", json!(5)).unwrap_err().contains("no span"));
        assert!(one("Nope", "div", "style.left", json!(5)).unwrap_err().contains("no component"));
    }

    #[test]
    fn next_names() {
        let d = std::env::temp_dir().join(format!("raea-tweak-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("pong-v2.tsx"), "").unwrap();
        assert_eq!(free_name(&d.join("pong.tsx")), d.join("pong-v3.tsx"));
        assert_eq!(free_name(&d.join("pong-v3.tsx")), d.join("pong-v4.tsx"));
        let _ = std::fs::remove_dir_all(&d);
    }
}
