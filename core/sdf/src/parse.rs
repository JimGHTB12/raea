//! The field written as calls, as raea's shape expressions are: `smooth_union(0.04, sphere(0.3), move(cone(0, 0, 0,
//! 0, 0.6, 0, 0.12, 0.05), 0, 0.2, 0))`. Numbers in metres and degrees.

use crate::node::{Node, Rot};

enum A {
    N(f64),
    S(Node),
}

impl A {
    fn shape(self) -> Option<Node> {
        if let A::S(s) = self { Some(s) } else { None }
    }
}

struct P<'a> {
    s: &'a [u8],
    i: usize,
}

impl P<'_> {
    fn ws(&mut self) {
        while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }
    fn eat(&mut self, c: u8) -> bool {
        self.ws();
        if self.s.get(self.i) == Some(&c) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn arg(&mut self) -> Result<A, String> {
        self.ws();
        let start = self.i;
        let c = *self.s.get(self.i).ok_or("sdf: the field ends too early")?;
        if c.is_ascii_digit() || c == b'-' || c == b'.' || c == b'+' {
            self.i += 1;
            while self.i < self.s.len() && (self.s[self.i].is_ascii_digit() || matches!(self.s[self.i], b'.' | b'e' | b'E') || (matches!(self.s[self.i], b'-' | b'+') && matches!(self.s[self.i - 1], b'e' | b'E'))) {
                self.i += 1;
            }
            let t = std::str::from_utf8(&self.s[start..self.i]).unwrap_or("");
            return t.parse().map(A::N).map_err(|_| format!("sdf: not a number: {t}"));
        }
        while self.i < self.s.len() && (self.s[self.i].is_ascii_alphanumeric() || self.s[self.i] == b'_') {
            self.i += 1;
        }
        let name = std::str::from_utf8(&self.s[start..self.i]).unwrap_or("").to_string();
        if name.is_empty() {
            return Err(format!("sdf: unexpected {:?}", c as char));
        }
        if !self.eat(b'(') {
            return Err(format!("sdf: {name} takes arguments: {name}(...)"));
        }
        let mut args = Vec::new();
        if !self.eat(b')') {
            loop {
                args.push(self.arg()?);
                if self.eat(b',') {
                    continue;
                }
                if self.eat(b')') {
                    break;
                }
                return Err(format!("sdf: {name}(...): a comma or a closing parenthesis"));
            }
        }
        node(&name, args).map(A::S)
    }
}

fn node(f: &str, a: Vec<A>) -> Result<Node, String> {
    let n: Vec<f64> = a.iter().filter_map(|x| if let A::N(v) = x { Some(*v) } else { None }).collect();
    let mut kids: Vec<Node> = a.into_iter().filter_map(A::shape).collect();
    let need = |k: usize| -> Result<(), String> { if n.len() < k { Err(format!("sdf: {f} needs {k} numbers, got {}", n.len())) } else { Ok(()) } };
    let one = |kids: &mut Vec<Node>| -> Result<Box<Node>, String> { if kids.is_empty() { Err(format!("sdf: {f} needs a field")) } else { Ok(Box::new(kids.remove(0))) } };
    let pos = |x: f64| -> Result<f64, String> { if x > 0.0 && x.is_finite() { Ok(x) } else { Err(format!("sdf: {f}: sizes must be above 0, got {x}")) } };
    Ok(match f {
        "sphere" => { need(1)?; Node::Sphere(pos(n[0])?) }
        "box" => { need(3)?; Node::Box([pos(n[0])?, pos(n[1])?, pos(n[2])?], n.get(3).copied().unwrap_or(0.0).clamp(0.0, n[0].min(n[1]).min(n[2]) / 2.0)) }
        "cylinder" => { need(2)?; Node::Cylinder(pos(n[0])?, pos(n[1])?) }
        "torus" => { need(2)?; Node::Torus(pos(n[0])?, pos(n[1])?) }
        "ellipsoid" => { need(3)?; Node::Ellipsoid([pos(n[0])?, pos(n[1])?, pos(n[2])?]) }
        // cone(ax, ay, az, bx, by, bz, ra [, rb]): rb = ra is a capsule
        "cone" | "capsule" => { need(7)?; Node::Cone([n[0], n[1], n[2]], [n[3], n[4], n[5]], n[6].max(0.0), n.get(7).copied().unwrap_or(n[6]).max(0.0)) }
        // tube(x, y, z, r, x, y, z, r, ...): two points or more
        "tube" => {
            if n.len() < 8 || n.len() % 4 != 0 {
                return Err(format!("sdf: tube takes x, y, z, radius for each of 2 or more points, got {} numbers", n.len()));
            }
            Node::Tube(n.chunks(4).map(|c| ([c[0], c[1], c[2]], c[3].max(0.0))).collect())
        }
        "union" => Node::Union(nonempty(f, kids)?),
        "smooth_union" => { need(1)?; Node::SmoothUnion(n[0].max(0.0), nonempty(f, kids)?) }
        "intersect" => Node::Intersect(nonempty(f, kids)?),
        "smooth_intersect" => { need(1)?; Node::SmoothIntersect(n[0].max(0.0), nonempty(f, kids)?) }
        "subtract" | "minus" => {
            if kids.len() < 2 { return Err(format!("sdf: {f}(a, b): two fields")); }
            let b = kids.remove(1);
            Node::Subtract(Box::new(kids.remove(0)), Box::new(b))
        }
        "smooth_subtract" | "smooth_minus" => {
            need(1)?;
            if kids.len() < 2 { return Err(format!("sdf: {f}(k, a, b): two fields")); }
            let b = kids.remove(1);
            Node::SmoothSubtract(n[0].max(0.0), Box::new(kids.remove(0)), Box::new(b))
        }
        "move" => { need(3)?; Node::Move(one(&mut kids)?, [n[0], n[1], n[2]]) }
        "rotate" => { need(3)?; Node::Rotate(one(&mut kids)?, Rot::degrees(n[0], n[1], n[2])) }
        "scale" => { need(1)?; Node::Scale(one(&mut kids)?, pos(n[0])?) }
        "mirror_x" => Node::MirrorX(one(&mut kids)?),
        "round" => { need(1)?; Node::Round(one(&mut kids)?, n[0]) }
        "shell" => { need(1)?; Node::Shell(one(&mut kids)?, pos(n[0])?) }
        // displace(field, amplitude, frequency [, octaves, seed])
        "displace" => { need(2)?; Node::Displace(one(&mut kids)?, n[0], n[1], n.get(2).copied().unwrap_or(4.0) as u32, n.get(3).copied().unwrap_or(1.0) as u32) }
        o => return Err(format!("sdf: no field {o}: sphere, box, cylinder, torus, ellipsoid, cone, capsule, tube, union, smooth_union, intersect, smooth_intersect, subtract, smooth_subtract, move, rotate, scale, mirror_x, round, shell, displace")),
    })
}

fn nonempty(f: &str, v: Vec<Node>) -> Result<Vec<Node>, String> {
    if v.is_empty() { Err(format!("sdf: {f} needs fields")) } else { Ok(v) }
}

pub fn parse(text: &str) -> Result<Node, String> {
    let mut p = P { s: text.as_bytes(), i: 0 };
    let a = p.arg()?;
    p.ws();
    if p.i < p.s.len() {
        return Err(format!("sdf: unexpected {:?} after the field", &text[p.i..(p.i + 12).min(text.len())]));
    }
    match a {
        A::S(n) => Ok(n),
        A::N(_) => Err("sdf: a field, not a number".into()),
    }
}
