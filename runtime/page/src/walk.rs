//! The walk's state across frames: each element's Want kept while its kind, props and text are unchanged.

use super::*;

/// The page's elements as the last walk made them, kept for the next (ZeroHeap: an element whose React kind, props
/// and text are as they were keeps its `Want`, and each parent's list keeps its room).
#[derive(Default)]
pub(crate) struct Walk {
    pub(crate) want: Vec<Want>,
    /// what each of `want`'s elements was made from, when it can be kept (None: made again each walk)
    pub(crate) made: Vec<Option<Made>>,
    /// how many of `want` this walk has
    pub(crate) len: usize,
    /// each parent's children in order
    pub(crate) order: HashMap<u64, Vec<u64>>,
    /// the parents this walk listed (`order`'s others go)
    pub(crate) listed: std::collections::HashSet<u64>,
    /// the elements every root starts with (`frame_wants`), for the size, brand kit and under element they were made for
    pub(crate) lead: Option<Lead>,
}

/// An element's React kind, props and text, as its `Want` was made from them.
pub(crate) struct Made {
    pub(crate) kind: String,
    pub(crate) props: Option<Map<String, Value>>,
    pub(crate) text: Option<String>,
}

pub(crate) struct Lead {
    pub(crate) key: (u32, u32, Option<String>, bool),
    pub(crate) want: Vec<Want>,
    pub(crate) order: HashMap<u64, Vec<u64>>,
    pub(crate) first: Vec<u64>,
}

/// Where an element's parent's path is: a string, or the parent's place in the walk's `want`.
#[derive(Clone, Copy)]
pub(crate) enum At<'a> {
    Str(&'a str),
    In(usize),
}

/// The end of an element's path after its parent's: its place among the parent's elements, a text among them, or its
/// own text.
#[derive(Clone, Copy)]
pub(crate) enum Tail {
    N(usize),
    T,
    DotT,
}

impl Tail {
    pub(crate) fn path(self, base: &str) -> String {
        match self {
            Tail::N(n) if base.is_empty() => n.to_string(),
            Tail::N(n) => format!("{base}.{n}"),
            Tail::T => format!("{base}t"),
            Tail::DotT => format!("{base}.t"),
        }
    }

    /// Whether `s` is `path(base)`, without writing it.
    pub(crate) fn is(self, s: &str, base: &str) -> bool {
        use std::fmt::Write;
        struct Cmp<'a>(&'a str);
        impl Write for Cmp<'_> {
            fn write_str(&mut self, t: &str) -> std::fmt::Result {
                self.0 = self.0.strip_prefix(t).ok_or(std::fmt::Error)?;
                Ok(())
            }
        }
        let mut c = Cmp(s);
        let ok = match self {
            Tail::N(n) if base.is_empty() => write!(c, "{n}"),
            Tail::N(n) => write!(c, "{base}.{n}"),
            Tail::T => write!(c, "{base}t"),
            Tail::DotT => write!(c, "{base}.t"),
        };
        ok.is_ok() && c.0.is_empty()
    }
}

impl Walk {
    /// A walk of a root `w` x `h` starting: its first elements (`frame_wants`'s, and the under element when `under`)
    pub(crate) fn begin(&mut self, w: u32, h: u32, brand: Option<&str>, under: bool) {
        self.len = 0;
        self.listed.clear();
        if !self.lead.as_ref().is_some_and(|l| l.key.0 == w && l.key.1 == h && l.key.2.as_deref() == brand && l.key.3 == under) {
            let (mut want, order, mut first) = frame_wants(w, h, brand);
            if under {
                let style = serde_json::json!({"position": "absolute", "left": 0, "top": 0, "width": w, "height": h, "zIndex": -1, "objectFit": "fill"});
                let mut props = Map::new();
                props.insert("style".into(), style);
                want.push(Want { id: UNDER, parent: BODY, kind: "video".into(), props, text: None, path: "under".into() });
                first.insert(0, UNDER);
            }
            self.lead = Some(Lead { key: (w, h, brand.map(str::to_string), under), want, order, first });
        }
        let lead = self.lead.take().unwrap();
        for w in &lead.want {
            let same = self.want.get(self.len).is_some_and(|x| x.id == w.id && x.parent == w.parent && x.kind == w.kind && x.props == w.props && x.text == w.text && x.path == w.path);
            if same {
                self.len += 1;
            } else {
                self.put(w.clone(), None);
            }
        }
        for (k, v) in &lead.order {
            let mut l = self.list(*k);
            l.extend_from_slice(v);
            self.order.insert(*k, l);
        }
        self.lead = Some(lead);
    }

    /// The walk done: the lead's sheets first among the body's children, the lists of parents gone dropped.
    pub(crate) fn end(&mut self) {
        let first = &self.lead.as_ref().unwrap().first;
        self.listed.insert(BODY);
        let l = self.order.entry(BODY).or_default();
        l.extend_from_slice(first);
        l.rotate_right(first.len());
        let listed = &self.listed;
        self.order.retain(|k, _| listed.contains(k));
    }

    pub(crate) fn wants(&self) -> &[Want] {
        &self.want[..self.len]
    }

    pub(crate) fn base<'a>(&'a self, at: At<'a>) -> &'a str {
        match at {
            At::Str(s) => s,
            At::In(i) => &self.want[i].path,
        }
    }

    /// `want`'s next element.
    pub(crate) fn put(&mut self, w: Want, made: Option<Made>) {
        if self.len < self.want.len() {
            self.want[self.len] = w;
            self.made[self.len] = made;
        } else {
            self.want.push(w);
            self.made.push(made);
        }
        self.len += 1;
    }

    /// `want`'s next element, its path `tail` after `at`'s, made by `make` from React's `kind`, `props` and `text`,
    /// or kept as it was when they are as they were. Its place in `want`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn element(&mut self, id: u64, parent: u64, at: At, tail: Tail, kind: &str, props: Option<&Map<String, Value>>, text: Option<&str>, make: impl FnOnce() -> Want) -> usize {
        let i = self.len;
        let kept = i < self.want.len() && {
            let w = &self.want[i];
            w.id == id && w.parent == parent && self.made[i].as_ref().is_some_and(|m| m.kind == kind && m.props.as_ref() == props && m.text.as_deref() == text) && tail.is(&w.path, self.base(at))
        };
        if kept {
            self.len += 1;
        } else {
            let path = tail.path(self.base(at));
            let mut w = make();
            w.path = path;
            // what React gave, kept to compare next frame: the slot's record brought up to date in place when it has one
            // (ZeroHeap: a style with one value moved copies nothing new)
            if let Some(Some(m)) = self.made.get_mut(i) {
                if m.kind != kind {
                    m.kind = kind.to_string();
                }
                match (&mut m.props, props) {
                    (Some(d), Some(s)) => raea_core_wire::tree::sync_map(d, s),
                    (d, s) => *d = s.cloned(),
                }
                match (&mut m.text, text) {
                    (Some(d), Some(s)) => {
                        if d != s {
                            d.clear();
                            d.push_str(s);
                        }
                    }
                    (d, s) => *d = s.map(str::to_string),
                }
                let m = self.made[i].take();
                self.put(w, m);
                return i;
            }
            self.put(w, Some(Made { kind: kind.to_string(), props: props.cloned(), text: text.map(str::to_string) }));
        }
        i
    }

    /// `parent`'s list of children, emptied (its room kept); put it back in `order` once filled.
    pub(crate) fn list(&mut self, parent: u64) -> Vec<u64> {
        self.listed.insert(parent);
        let mut l = self.order.remove(&parent).unwrap_or_default();
        l.clear();
        l
    }
}

/// The elements every root starts with: a body `w` x `h` in the default face, the page's own sheet and the brand kit;
/// and the sheets' ids, to go first among the body's children.
pub(crate) fn frame_wants(w: u32, h: u32, brand: Option<&str>) -> (Vec<Want>, HashMap<u64, Vec<u64>>, Vec<u64>) {
    let mut want = Vec::new();
    let mut order: HashMap<u64, Vec<u64>> = HashMap::new();
    let body = serde_json::json!({"style": {"position": "absolute", "left": 0, "top": 0, "width": w, "height": h, "margin": 0, "fontFamily": DEFAULT_FAMILY}});
    want.push(Want { id: BODY, parent: 0, kind: "div".into(), props: body.as_object().cloned().unwrap_or_default(), text: None, path: "body".into() });
    order.insert(0, vec![BODY]);
    let mut first = vec![BASE];
    want.push(Want { id: BASE, parent: BODY, kind: "style".into(), props: Map::new(), text: None, path: "base".into() });
    want.push(Want { id: BASE | TEXT_BIT, parent: BASE, kind: "text".into(), props: Map::new(), text: Some(BASE_CSS.into()), path: "base.t".into() });
    order.insert(BASE, vec![BASE | TEXT_BIT]);
    if let Some(css) = brand {
        want.push(Want { id: BRAND, parent: BODY, kind: "style".into(), props: Map::new(), text: None, path: "brand".into() });
        want.push(Want { id: BRAND | TEXT_BIT, parent: BRAND, kind: "text".into(), props: Map::new(), text: Some(css.to_string()), path: "brand.t".into() });
        order.insert(BRAND, vec![BRAND | TEXT_BIT]);
        first.push(BRAND);
    }
    (want, order, first)
}

/// A <Screen>'s nested composition, taken out of the page: its element (`top`, at the root's top left) and everything
/// under it, laid out at the screen's size in a root of its own.
pub(crate) struct ScreenTree {
    pub(crate) name: String,
    pub(crate) w: u32,
    pub(crate) h: u32,
    pub(crate) top: u64,
    pub(crate) want: Vec<Want>,
    pub(crate) order: HashMap<u64, Vec<u64>>,
    pub(crate) videos: Vec<VideoAt>,
}

// the camera view's box, kept in raea-runtime-react beside the native call that reads it
pub use raea_runtime_react::VIEW_BOX;

thread_local! {
    /// the page id of the layered Canvas3D's element the last walk met (the scene's slot)
    pub(crate) static SCENE_SLOT: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
    /// the page element holding the first camera the last walk met outside the 3D's parts (its view's box)
    pub(crate) static VIEW_HOLDER: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
    /// how deep the walk is inside 3D parts
    pub(crate) static IN_3D: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// whether the walk is inside an <svg> (not back in HTML under its <foreignObject>): a `text`, `image` or `title`
    /// there is SVG's, as sdk/ui's host says with `$ns`, so its attributes keep their SVG names (svg_owned)
    pub(crate) static IN_SVG: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// the app screens (`<Screen app>`) the last walk met: each one's name, size and module
    pub(crate) static APP_SCREENS: std::cell::RefCell<Vec<(String, u32, u32, String)>> = const { std::cell::RefCell::new(Vec::new()) };
    /// why the last walk's page can't be rendered as written (an <img> of a file that isn't there, `page_props`; a take
    /// framing="face" can't frame, `video_fit`): a render fails with them
    pub(crate) static REFUSED: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
    /// the texture elements (`data-raea-texture`) the last walk met: each one's texture id and its page id
    pub(crate) static TEXTURE_SLOTS: std::cell::RefCell<Vec<(u64, u64)>> = const { std::cell::RefCell::new(Vec::new()) };
    /// the textures shared for texture elements to show (`share_texture`), by id: each one's texture and size
    pub(crate) static SHARED: std::cell::RefCell<HashMap<u64, (wgpu::Texture, u32, u32)>> = std::cell::RefCell::new(HashMap::new());
}

/// Texture `id` (`w` x `h`, on the shared device) shown by every `<img data-raea-texture={id}>` on a page synced on this
/// thread after it: a window's chrome drawing the stage in its own frame (the old shell's stage).
pub fn share_texture(id: u64, t: wgpu::Texture, w: u32, h: u32) {
    SHARED.with(|s| s.borrow_mut().insert(id, (t, w, h)));
}

/// Kinds that never draw on the page (3D, sound, scripts and the page's own root).
pub(crate) fn undrawn(kind: &str) -> bool {
    matches!(kind, "mesh" | "light" | "camera" | "text3d" | "surface" | "#root" | "audio" | "script")
}
