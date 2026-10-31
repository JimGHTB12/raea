//! What a compiled animation gives at a frame: each property's value, ready to apply.

use serde_json::{Map, Value};

/// What one frame of an animation sets.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sampled {
    pub opacity: Option<f64>,
    pub x: Option<(f64, String)>,
    pub y: Option<(f64, String)>,
    pub z: Option<f64>,
    pub scale_x: Option<f64>,
    pub scale_y: Option<f64>,
    pub rotate: Option<f64>,
    pub rotate_x: Option<f64>,
    pub rotate_y: Option<f64>,
    pub width: Option<(f64, String)>,
    pub height: Option<(f64, String)>,
    pub clip: [Option<f64>; 4],
    pub blur: Option<f64>,
    pub text: Option<String>,
    /// colour properties and their colour for the page (rgba() or color-mix()) and, when known, for 3D (#rrggbb)
    pub colors: Vec<(String, String, Option<String>)>,
    pub numbers: Vec<(String, f64)>,
    /// outside its show window: not laid out (display none)
    pub hidden: bool,
}

/// A counter's text: `v` with `digits` decimals (no "-0.00"), its thousands split by `separator`, between `prefix` and
/// `suffix` (raea's sampleAnimation writes the same)
pub fn counter_text(v: f64, digits: u32, prefix: &str, suffix: &str, separator: &str) -> String {
    let txt = format!("{:.*}", digits.min(6) as usize, v);
    let txt = if txt.trim_start_matches('-').chars().all(|c| c == '0' || c == '.') { txt.trim_start_matches('-').to_string() } else { txt };
    let (sign, rest) = match txt.strip_prefix('-') {
        Some(r) => ("-", r),
        None => ("", txt.as_str()),
    };
    let (int, frac) = rest.split_once('.').map_or((rest, None), |(i, f)| (i, Some(f)));
    let mut grouped = String::with_capacity(rest.len() + 4);
    for (i, c) in int.chars().enumerate() {
        if i > 0 && !separator.is_empty() && (int.len() - i) % 3 == 0 {
            grouped.push_str(separator);
        }
        grouped.push(c);
    }
    match frac {
        Some(f) => format!("{prefix}{sign}{grouped}.{f}{suffix}"),
        None => format!("{prefix}{sign}{grouped}{suffix}"),
    }
}

/// A number as CSS writes it: no "-0", no exponent for tiny values.
pub fn num(v: f64) -> String {
    let v = if v.abs() < 1e-9 { 0.0 } else { v };
    let r = (v * 1e6).round() / 1e6;
    format!("{r}")
}

impl Sampled {
    /// The page's style keys (React's camelCase names, as the DOM mirror's style objects hold them). Transforms go to
    /// the individual `translate`, `scale` and `rotate` properties, so an element's own `transform` stays its own.
    pub fn css(&self) -> Map<String, Value> {
        let mut m = Map::new();
        let n = |v: f64| serde_json::Number::from_f64(if v.abs() < 1e-9 { 0.0 } else { (v * 1e6).round() / 1e6 }).map(Value::Number).unwrap_or(Value::from(0));
        if let Some(o) = self.opacity {
            m.insert("opacity".into(), n(o.clamp(0.0, 1.0)));
        }
        if self.x.is_some() || self.y.is_some() || self.z.is_some() {
            let (x, ux) = self.x.clone().unwrap_or((0.0, "px".into()));
            let (y, uy) = self.y.clone().unwrap_or((0.0, "px".into()));
            let mut t = format!("{}{ux} {}{uy}", num(x), num(y));
            if let Some(z) = self.z {
                t.push_str(&format!(" {}px", num(z)));
            }
            m.insert("translate".into(), Value::String(t));
        }
        if self.scale_x.is_some() || self.scale_y.is_some() {
            m.insert("scale".into(), Value::String(format!("{} {}", num(self.scale_x.unwrap_or(1.0)), num(self.scale_y.unwrap_or(1.0)))));
        }
        if let Some(r) = self.rotate {
            m.insert("rotate".into(), Value::String(format!("{}deg", num(r))));
        }
        if let Some((w, u)) = &self.width {
            m.insert("width".into(), Value::String(format!("{}{u}", num(w.max(0.0)))));
        }
        if let Some((h, u)) = &self.height {
            m.insert("height".into(), Value::String(format!("{}{u}", num(h.max(0.0)))));
        }
        if self.clip.iter().any(Option::is_some) {
            let c = self.clip.map(|v| num(v.unwrap_or(0.0).clamp(0.0, 100.0)));
            m.insert("clipPath".into(), Value::String(format!("inset({}% {}% {}% {}%)", c[0], c[1], c[2], c[3])));
        }
        if let Some(b) = self.blur {
            m.insert("filter".into(), Value::String(format!("blur({}px)", num(b))));
        }
        for (k, css, _) in &self.colors {
            m.insert(k.clone(), Value::String(css.clone()));
        }
        for (k, v) in &self.numbers {
            m.insert(k.clone(), n(*v));
        }
        m
    }

    /// A 3D node's props (its data-props) with what this frame sets: x, y, z its position, rotateX/Y and rotate its
    /// rotation (radians, as the 3D props take them), scale (a number when uniform), opacity, colours as #rrggbb and
    /// any other number under its own name. True when anything changed.
    pub fn patch_3d(&self, props: &mut Map<String, Value>) -> bool {
        let before = props.clone();
        let n = |v: f64| serde_json::Number::from_f64(v).map(Value::Number).unwrap_or(Value::from(0));
        let vec3 = |props: &Map<String, Value>, k: &str, d: f64| -> [f64; 3] {
            match props.get(k) {
                Some(Value::Array(a)) => std::array::from_fn(|i| a.get(i).and_then(Value::as_f64).unwrap_or(d)),
                Some(Value::Number(x)) => [x.as_f64().unwrap_or(d); 3],
                _ => [d; 3],
            }
        };
        if self.x.is_some() || self.y.is_some() || self.z.is_some() {
            let mut p = vec3(props, "position", 0.0);
            if let Some((x, _)) = &self.x {
                p[0] = *x;
            }
            if let Some((y, _)) = &self.y {
                p[1] = *y;
            }
            if let Some(z) = self.z {
                p[2] = z;
            }
            props.insert("position".into(), Value::Array(p.iter().map(|v| n(*v)).collect()));
        }
        if self.rotate_x.is_some() || self.rotate_y.is_some() || self.rotate.is_some() {
            let mut r = vec3(props, "rotation", 0.0);
            for (i, v) in [self.rotate_x, self.rotate_y, self.rotate].into_iter().enumerate() {
                if let Some(v) = v {
                    r[i] = v;
                }
            }
            props.insert("rotation".into(), Value::Array(r.iter().map(|v| n(*v)).collect()));
        }
        if self.scale_x.is_some() || self.scale_y.is_some() {
            let (sx, sy) = (self.scale_x.unwrap_or(1.0), self.scale_y.unwrap_or(1.0));
            props.insert("scale".into(), if sx == sy { n(sx.max(1e-6)) } else { Value::Array(vec![n(sx.max(1e-6)), n(sy.max(1e-6)), n(1.0)]) });
        }
        if let Some(o) = self.opacity {
            props.insert("opacity".into(), n(o.clamp(0.0, 1.0)));
        }
        for (k, css, hex) in &self.colors {
            props.insert(k.clone(), Value::String(hex.clone().unwrap_or_else(|| css.clone())));
        }
        for (k, v) in &self.numbers {
            props.insert(k.clone(), n(*v));
        }
        *props != before
    }
}
