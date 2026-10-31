//! An edit's image rows (the pages raea painted, each on the frames it shows) drawn by Blitz: one `<img>` per
//! row on a raea-page page, the row's values at the frame as its inline style, over transparent. The engine's overlay
//! (engine.rs), the direct render's (gpuview.rs) and the CPU's (cpuview.rs) all draw them here.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use raea_core_wire::tree::Want;
use raea_page::motion::BlitzPage as Page;
use serde_json::{Map, Value, json};

type R<T> = Result<T, String>;

const ROOT: u64 = 1;
/// a row's element is FIRST + its index
const FIRST: u64 = 16;

struct Row {
    /// frames it shows on, `to` exclusive
    from: i64,
    to: i64,
    /// the image's file
    src: Option<PathBuf>,
    /// its props as installed (its constants)
    base: Map<String, Value>,
}

/// The image rows of one edit, drawn on their own Blitz page.
pub struct Rows {
    page: Page,
    rows: Vec<Row>,
    by_id: HashMap<String, usize>,
    /// each row's props at the frame (its base where nothing set them)
    props: Vec<Map<String, Value>>,
    pub w: u32,
    pub h: u32,
    frames: i64,
    /// what the page last drew: the frame's visible rows and their props, so an unchanged frame isn't drawn again
    drawn: Option<Vec<(usize, Map<String, Value>)>>,
    rgba: Vec<u8>,
}

impl Rows {
    /// The image rows of an edit (engine.rs row2d: {id, kind, props, from?, to?}), `w` x `h`, `frames` long; assets
    /// resolve against `dir`. On the shared GPU device, so the page's picture is a texture the engine can draw.
    pub fn new(instances: &[Value], w: u32, h: u32, frames: i64, dir: &Path) -> R<Rows> {
        let mut by_id = HashMap::new();
        let mut rows = Vec::new();
        for (i, r) in instances.iter().enumerate() {
            let id = r["id"].as_str().ok_or(format!("image row {i}: no id"))?.to_string();
            let base = r["props"].as_object().cloned().unwrap_or_default();
            let src = base.get("src").and_then(|s| s.get("asset")).and_then(Value::as_str).map(|a| dir.join(a));
            by_id.insert(id, i);
            rows.push(Row { from: r["from"].as_i64().unwrap_or(0), to: r["to"].as_i64().unwrap_or(frames), src, base });
        }
        let props = rows.iter().map(|r| r.base.clone()).collect();
        let mut page = Page::new_on(w, h, None, Some(crate::device::render_creation()))?;
        page.not_dumped();
        Ok(Rows { page, rows, by_id, props, w, h, frames, drawn: None, rgba: Vec::new() })
    }

    /// Row `id`'s props as installed.
    pub fn base(&self, id: &str) -> Option<&Map<String, Value>> {
        self.by_id.get(id).map(|&i| &self.rows[i].base)
    }

    /// Row `id`'s props for the frames drawn from now.
    pub fn set(&mut self, id: &str, props: Map<String, Value>) {
        if let Some(&i) = self.by_id.get(id) {
            self.props[i] = props;
        }
    }

    pub fn frames(&self) -> i64 {
        self.frames
    }

    /// Frame `f` drawn (unless it's what was drawn last); its picture is `texture`, or `rgba`.
    pub fn draw(&mut self, f: i64) -> bool {
        let shown: Vec<(usize, Map<String, Value>)> = (0..self.rows.len()).filter(|&i| f >= self.rows[i].from && f < self.rows[i].to).map(|i| (i, self.props[i].clone())).collect();
        if self.drawn.as_ref() == Some(&shown) {
            return false;
        }
        let root = json!({"style": {"position": "absolute", "left": 0, "top": 0, "width": self.w, "height": self.h, "overflow": "hidden"}});
        let mut want = vec![Want { id: ROOT, parent: 0, kind: "div".into(), props: root.as_object().cloned().unwrap_or_default(), text: None, path: "root".into() }];
        let mut order: HashMap<u64, Vec<u64>> = HashMap::new();
        order.insert(0, vec![ROOT]);
        for (i, props) in &shown {
            let id = FIRST + *i as u64;
            let mut p = Map::new();
            p.insert("style".into(), Value::Object(style_for(props)));
            if let Some(src) = &self.rows[*i].src {
                p.insert("src".into(), Value::String(src.display().to_string()));
            }
            want.push(Want { id, parent: ROOT, kind: "img".into(), props: p, text: None, path: format!("r{i}") });
            order.entry(ROOT).or_default().push(id);
        }
        self.page.sync(&want, &order);
        self.page.draw();
        self.drawn = Some(shown);
        self.rgba.clear();
        true
    }

    /// The last frame drawn, read back: premultiplied RGBA, `w` x `h`.
    pub fn rgba(&mut self) -> R<&[u8]> {
        if self.rgba.is_empty() {
            self.rgba = self.page.frame()?;
        }
        Ok(&self.rgba)
    }

    /// Frame `f`, drawn and read back (premultiplied RGBA) into `out`.
    pub fn render(&mut self, f: i64, out: &mut Vec<u8>) -> R<()> {
        self.draw(f);
        out.clear();
        out.extend_from_slice(self.rgba()?);
        Ok(())
    }

    /// The last frame drawn: its texture on the shared device (`Rgba8UnormSrgb`, premultiplied).
    pub fn texture(&self) -> Option<wgpu::Texture> {
        self.page.texture().map(|t| (*t).clone())
    }
}

fn px(v: &Value) -> Option<Value> {
    v.as_f64().map(|n| json!(format!("{}px", (n * 1000.0).round() / 1000.0)))
}

/// An image row's values as its inline style (numbers are CSS px).
pub fn style_for(props: &Map<String, Value>) -> Map<String, Value> {
    let mut css = Map::new();
    css.insert("position".into(), json!("absolute"));
    css.insert("objectFit".into(), json!("cover"));
    let mut transform = Vec::new();
    for (k, v) in props {
        let (key, val) = match k.as_str() {
            "x" => ("left", px(v)),
            "y" => ("top", px(v)),
            "w" => ("width", px(v)),
            "h" => ("height", px(v)),
            "opacity" => ("opacity", v.as_f64().map(|o| json!(o.clamp(0.0, 1.0)))),
            "radius" => ("borderRadius", px(v)),
            "fit" => ("objectFit", v.as_str().map(|x| json!(x))),
            // rdb hands numbers over as floats (-1.0)
            "z" => ("zIndex", v.as_f64().map(|z| json!(z.round() as i64))),
            "rotate" => {
                if let Some(d) = v.as_f64() {
                    transform.push(format!("rotate({d}deg)"));
                }
                continue;
            }
            "scale" => {
                if let Some(s) = v.as_f64() {
                    transform.push(format!("scale({s})"));
                }
                continue;
            }
            _ => continue,
        };
        if let Some(val) = val {
            css.insert(key.into(), val);
        }
    }
    if !transform.is_empty() {
        css.insert("transform".into(), json!(transform.join(" ")));
    }
    css
}
