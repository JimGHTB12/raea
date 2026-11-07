//! The tree drawn as Rune elements (R2a): each wire element is a Rune box keyed by its wire id ("w12"), each text node
//! a Rune text with the font props it inherits, and each style prop a builder call (css.rs reads them). The host's
//! defaults are Tailwind's preflight, the base TSX + Tailwind apps run on: border-box, no margins or borders, controls
//! inheriting font, line-height 1.5, black text at 16px in Inter. Rune has no block or inline layout, so a block is a
//! stretched column and a run of inline children a wrapping row; where that differs from CSS it is reported.

use std::collections::HashMap;

use rune::{Color, El, ScrollSpec, Shadow, Spring, TextEdit, TextStyle, Transition, col, el, row, text};
use serde_json::{Map, Value};

use crate::css::{self, Align, Css, Display, Len, Overflow};
use crate::images::Images;
use crate::report::Report;
use crate::tree::{Node, ROOT, Tree};

/// What text inherits from its ancestors (CSS's inherited font props).
#[derive(Clone, Debug)]
struct Inherit {
    color: Color,
    size: f32,
    weight: f32,
    family: Option<&'static str>,
    line: Len,
    tracking: f32,
    align: Align,
    italic: bool,
    /// the strut: the line height of the nearest element that isn't laid out inline (CSS: every line box in a block is at
    /// least as tall as the block's own line height, whatever its inline content's)
    strut: f32,
    /// the parent is a flex container, so this element is a flex item: blockified, inline or not
    flex_parent: bool,
}

impl Default for Inherit {
    fn default() -> Self {
        Inherit { color: Color::BLACK, size: 16.0, weight: 400.0, family: None, line: Len::Pct(150.0), tracking: 0.0, align: Align::Start, italic: false, strut: 24.0, flex_parent: false }
    }
}

/// An element's Rune key: its wire id.
pub fn key(id: u64) -> String {
    format!("w{id}")
}

/// The page under the root as one Rune element filling the viewport, as the document's body does.
/// What a build reports to and loads from.
pub struct Ctx<'a> {
    pub report: &'a mut Report,
    pub images: &'a Images,
    pub videos: &'a raea_rune_media::video::Videos,
    /// the page's text fields' edits, by wire id
    pub edits: &'a HashMap<u64, TextEdit>,
    /// each element's padding and border widths as mapped (l t r b, l t r b): the layout snapshot's
    pub boxes: &'a mut HashMap<u64, [f32; 8]>,
    /// each element's mapped style and text colour (Page::looks)
    pub looks: &'a mut HashMap<u64, (Css, Color)>,
    /// unrounded text widths (measure.rs)
    pub measure: &'a crate::measure::Measure,
    /// elements built so far that asked for Rune motion (runeprops.rs): a subtree that adds none lands at once
    pub marks: usize,
    /// what Tailwind's variants test (state.rs)
    pub live: crate::state::Live<'a>,
}

pub fn build(tree: &Tree, cx: &mut Ctx) -> El {
    let root = tree.get(ROOT).expect("the root is always there");
    let inh = Inherit::default();
    // the canvas is white, as a browser's is: Rune clears the window with the root's fill
    let e = col().key(key(ROOT)).w_full().h_full().items_stretch().fill(Color::WHITE).children(kids(tree, root, &inh, "", cx));
    let e = crate::events::attach(e, ROOT, root, true).transition(Transition::none()).motion(Spring::NONE);
    if cx.marks == 0 { e.instant_changes() } else { e }
}

fn kids(tree: &Tree, n: &Node, inh: &Inherit, path: &str, cx: &mut Ctx) -> Vec<El> {
    let mut out = Vec::new();
    for (i, c) in n.children.iter().enumerate() {
        let Some(child) = tree.get(*c) else { continue };
        let p = format!("{path}/{}[{i}]", child.kind);
        if let Some(e) = node(tree, *c, child, inh, &p, cx) {
            out.push(e);
        }
    }
    out
}

/// The display a tag has when its style doesn't say (Chrome's defaults, as preflight leaves them).
fn default_display(kind: &str) -> Display {
    match kind {
        "span" | "a" | "b" | "i" | "em" | "strong" | "label" | "code" | "small" | "button" | "editableText" | "img" | "video" | "text" => Display::Inline,
        _ => Display::Block,
    }
}

/// The element's style props: its Tailwind classes' (R2c) under its inline style, which wins as CSS has it (an inline
/// shorthand also overrides the classes' longhands of it), and the utilities nothing knew.
fn style_of(id: u64, n: &Node, live: &crate::state::Live) -> (Map<String, Value>, Vec<String>) {
    let inline = n.props.get("style").and_then(Value::as_object).cloned().unwrap_or_default();
    let Some(class) = n.props.get("className").and_then(Value::as_str) else { return (inline, Vec::new()) };
    let (mut out, unknown) = crate::tailwind::style_in(class, &live.of(id, n));
    for (k, v) in inline {
        let longhand = |l: &String| match k.as_str() {
            "padding" | "margin" => l.starts_with(k.as_str()),
            "gap" => l == "rowGap" || l == "columnGap",
            "borderWidth" => l.starts_with("border") && l.ends_with("Width"),
            "borderRadius" => l.starts_with("border") && l.ends_with("Radius"),
            "overflow" => l.starts_with("overflow"),
            _ => false,
        };
        out.retain(|l, _| !longhand(l));
        out.insert(k, v);
    }
    (out, unknown)
}

fn display_of(id: u64, n: &Node, live: &crate::state::Live) -> Display {
    css::parse(&style_of(id, n, live).0).0.display.unwrap_or(default_display(&n.kind))
}

fn node(tree: &Tree, id: u64, n: &Node, inh: &Inherit, path: &str, cx: &mut Ctx) -> Option<El> {
    if n.kind == "text" {
        return Some(text_el(n.text.as_deref().unwrap_or(""), inh, cx));
    }
    let (style, unknown) = style_of(id, n, &cx.live);
    for u in unknown {
        cx.report.note(path, "className", &u, "a Tailwind utility the host doesn't map");
    }
    let (c, bad) = css::parse(&style);
    for b in bad {
        cx.report.add(path, b);
    }
    props(n, path, cx.report);
    let display = c.display.unwrap_or(default_display(&n.kind));
    if display == Display::None {
        return None;
    }
    if n.kind == "style" {
        let css: String = n.children.iter().filter_map(|k| tree.get(*k)?.text.clone()).collect();
        raea_rune_media::view::sheet(&css, path, cx.report);
        return None;
    }
    raea_rune_media::view::check_family(c.font_family.as_deref(), path, cx.report);
    let mut inh = inherit(inh, &c);
    cx.looks.insert(id, (c.clone(), inh.color));
    // a new strut where a block (or a flex item, or a button or field: inline-blocks) holds the lines; inline elements
    // keep their block's
    if display != Display::Inline || inh.flex_parent || matches!(n.kind.as_str(), "button" | "editableText" | "img" | "video") {
        inh.strut = line_px(&inh);
    }
    inh.flex_parent = display == Display::Flex;
    let before = cx.marks;
    let kit = n.props.get("data-rune-kit").and_then(Value::as_str).and_then(|j| crate::kit::build(id, j, inh.color, text_style(&inh), path, cx.report));
    let is_kit = kit.is_some();
    let mut e = if let Some(k) = kit {
        // the kit's element inside a row of the stand-in's own: the span's style and the DOM's no-motion defaults go
        // on the row, so they don't clobber the kit's padding (RollingText's edge fades sit in it), its springs, or
        // its rolling (a kit moves as Rune's does, so its subtree isn't made instant). The stand-in's role and
        // events stay on the kit's element, where a press lands.
        cx.marks += 1;
        row().child(crate::events::attach(a11y(k, n), id, n, false))
    } else {
        match (n.kind.as_str(), display) {
        ("editableText", _) => field(id, n, &inh, cx),
        ("img", _) => raea_rune_media::view::img(id, &n.props, c.fit, path, cx.images, cx.report),
        ("video", _) => raea_rune_media::view::video(&n.props, (id, c.fit), path, cx.videos, cx.report),
        ("button", _) => row().items_center().justify_center().children(kids(tree, n, &inh, path, cx)),
        (_, Display::Flex) => {
            let b = if c.column == Some(true) { col() } else { row() };
            let b = if c.wrap { b.flex_wrap() } else { b };
            b.children(kids(tree, n, &inh, path, cx))
        }
        (_, Display::Inline) => row().flex_wrap().children(kids(tree, n, &inh, path, cx)),
        (_, _) => block(tree, n, &inh, path, cx),
        }
    };
    e = e.key(key(id));
    if !is_kit {
        e = a11y(e, n);
        e = crate::events::attach(e, id, n, false);
    }
    // Rune's own props; without a transition of its own an element shows whole the frame it mounts, as the DOM does,
    // and a subtree with no motion in it lands every change at once
    let (mut own_transition, mut own_motion) = (false, false);
    if let Some(j) = n.props.get("data-rune").and_then(Value::as_str) {
        let (x, animated) = crate::runeprops::apply(e, j, path, cx.report);
        e = x;
        cx.marks += usize::from(animated);
        own_transition = j.contains("\"transition\"");
        own_motion = j.contains("\"motion\"");
    }
    if !own_transition {
        e = e.transition(Transition::none());
    }
    // CSS transforms and transitions (T3c): an element with a transition animates, so its subtree isn't instant
    let (x, animated) = crate::motion::apply(e, &c.motion);
    e = x;
    cx.marks += usize::from(animated);
    own_motion |= animated && c.motion.transition.as_ref().is_some_and(|t| t.props.colours || t.props.shadow);
    // and its own changes, colours too, land at once unless it has a spring of its own (CSS changes without a transition)
    if !own_motion {
        e = e.motion(Spring::NONE);
    }
    if cx.marks == before {
        e = e.instant_changes();
    }
    let (pad, bw) = padding(&c);
    cx.boxes.insert(id, [pad[3], pad[0], pad[1], pad[2], bw[3], bw[0], bw[1], bw[2]]);
    // its outline (T3e): its own, or Chrome's focus-visible ring on the page's focus
    if let Some((w, off, col)) = crate::outline::drawn(&c.outline, inh.color, cx.live.focus == Some(id) && cx.live.focus_visible) {
        e = e.outline(w, off, col);
    }
    Some(boxed(e, &c, &inh, display == Display::Flex, path, cx.report))
}

/// A block's children: a stretched column, or a wrapping row when every child is inline (CSS's line boxes, as near
/// as flex comes).
fn block(tree: &Tree, n: &Node, inh: &Inherit, path: &str, cx: &mut Ctx) -> El {
    let live = cx.live;
    let inline: Vec<bool> = n.children.iter().filter_map(|c| Some((*c, tree.get(*c)?))).map(|(c, k)| k.kind == "text" || display_of(c, k, &live) == Display::Inline).collect();
    let all = !inline.is_empty() && inline.iter().all(|i| *i);
    if all && inline.len() > 1 {
        cx.report.note(path, "inline layout", &format!("{} inline children", inline.len()), "inline formatting drawn as a wrapping row");
    } else if !all && inline.iter().any(|i| *i) {
        cx.report.note(path, "inline layout", "inline and block children mixed", "each inline child drawn on its own line");
    }
    let b = if all { row().flex_wrap() } else { col().items_stretch() };
    b.children(kids(tree, n, inh, path, cx))
}

/// A text field (R3 makes it editable): its value, or its placeholder at half the text's colour (preflight's).
fn field(id: u64, n: &Node, inh: &Inherit, cx: &mut Ctx) -> El {
    let blank = TextEdit::new(n.props.get("value").and_then(Value::as_str).unwrap_or(""));
    let edit = cx.edits.get(&id).unwrap_or(&blank);
    let placeholder = n.props.get("placeholder").and_then(Value::as_str).unwrap_or("");
    let input = crate::events::field(id, edit, text_style(inh), inh.color, placeholder).grow(1.0);
    row().items_center().child(input)
}

/// The element's own line height in points (0: Rune's default, for `normal`), in Chrome's 1/64 px layout units: a
/// ratio's float error (Tailwind's text-xs, 12 × calc(1 / 0.75)) stays 16, where Rune would round 16.000001 up to 17.
fn line_px(inh: &Inherit) -> f32 {
    let l = match inh.line {
        Len::Px(p) => p,
        Len::Pct(p) => inh.size * p / 100.0,
        Len::Auto => 0.0,
    };
    (l * 64.0).round() / 64.0
}

fn text_style(inh: &Inherit) -> TextStyle {
    // each line at least the strut tall, as a line box is
    let line = line_px(inh).max(inh.strut);
    let family = raea_rune_media::view::family(inh.family);
    TextStyle { family, line_height: line, tracking: inh.tracking, italic: inh.italic, ..TextStyle::new(inh.size, inh.weight) }
}

fn text_el(s: &str, inh: &Inherit, cx: &Ctx) -> El {
    let style = text_style(inh);
    let a = match inh.align {
        Align::Center => rune::Align::Center,
        Align::End => rune::Align::End,
        _ => rune::Align::Start,
    };
    // inert: a text node never listens, so presses and hovers go to the element under it, as DOM events target the
    // element around a text (Rune ends a press at plain text that isn't selectable)
    let e = text(s).font(style).color(inh.color).wrap().align(a).inert().transition(Transition::none());
    // Rune rounds a text's width up to a whole pixel; a browser doesn't (measure.rs). The unrounded width as the text's own,
    // which a container's intrinsic size reads (a max width it ignores), shrinkable so a short line still wraps.
    // Start-aligned text paints the same in that box; centred and end-aligned text keep Rune's width
    if matches!(inh.align, Align::Center | Align::End) { e } else { e.w(cx.measure.width(s, style)).shrink() }
}

fn inherit(inh: &Inherit, c: &Css) -> Inherit {
    let mut o = inh.clone();
    if let Some(v) = c.color {
        o.color = v;
    }
    if let Some(v) = c.font_size {
        o.size = v;
    }
    if let Some(v) = c.font_weight {
        o.weight = v;
    }
    if let Some(f) = &c.font_family {
        o.family = Some(intern(f));
    }
    if let Some(v) = c.line_height {
        o.line = v;
    }
    if let Some(v) = c.letter_spacing {
        o.tracking = v;
    }
    if let Some(v) = c.text_align {
        o.align = v;
    }
    if let Some(v) = c.italic {
        o.italic = v;
    }
    o
}

/// A family name as Rune's `Family::Named` wants it: 'static, each name leaked once.
fn intern(s: &str) -> &'static str {
    use std::sync::{Mutex, OnceLock};
    static NAMES: OnceLock<Mutex<Vec<&'static str>>> = OnceLock::new();
    let mut names = NAMES.get_or_init(Default::default).lock().unwrap();
    if let Some(n) = names.iter().find(|n| **n == s) {
        return n;
    }
    let n: &'static str = Box::leak(s.to_string().into_boxed_str());
    names.push(n);
    n
}

/// Props other than `style` that the host reads, and a report for the rest.
fn props(n: &Node, path: &str, report: &mut Report) {
    for k in n.props.keys() {
        let known = k == "style" || k == "className" || k.starts_with("data-") || k.starts_with("aria-") || matches!(k.as_str(), "role" | "id" | "tabIndex" | "value" | "placeholder" | "disabled" | "type" | "name" | "src" | "alt" | "autoplay" | "autoPlay" | "loop" | "muted" | "playsInline" | "playsinline" | "__seek");
        if !known {
            report.add(path, css::Unmapped { prop: k.clone(), value: n.props[k].to_string(), why: "a prop the Rune host doesn't read" });
        }
    }
}

const ROLES: [&str; 20] = [
    "button", "switch", "checkbox", "textbox", "link", "list", "listitem", "listbox", "option", "tab", "tablist", "dialog", "img", "heading", "group", "slider", "menu", "menuitem", "progressbar", "radio",
];

fn a11y(mut e: El, n: &Node) -> El {
    let role = n.props.get("role").and_then(Value::as_str).or(match n.kind.as_str() {
        "button" => Some("button"),
        "editableText" => Some("textbox"),
        "a" => Some("link"),
        "img" => Some("img"),
        _ => None,
    });
    if let Some(r) = role.and_then(|r| ROLES.iter().find(|x| **x == r)) {
        e = e.role(r);
    }
    if let Some(l) = n.props.get("aria-label").and_then(Value::as_str) {
        e = e.label(l);
    }
    if let Some(c) = n.props.get("aria-checked") {
        e = e.checked(c == true || c == "true");
    }
    if n.props.get("disabled").is_some_and(|d| d == true) {
        e = e.disabled(true);
    }
    e
}

/// The box props: size, spacing, position, paint.
/// The padding Rune lays out with, top, right, bottom, left, and the border widths: CSS borders take room inside the
/// border box, and Rune's border is paint only, so their widths join the padding
fn padding(c: &Css) -> ([f32; 4], [f32; 4]) {
    let bw = c.border_width.map(|w| w.unwrap_or(0.0));
    (std::array::from_fn(|i| c.padding[i].unwrap_or(0.0) + bw[i]), bw)
}

fn boxed(mut e: El, c: &Css, inh: &Inherit, flex: bool, path: &str, report: &mut Report) -> El {
    let mut note = |what: &str, v: &Len, why: &'static str| report.note(path, what, &format!("{v:?}"), why);
    for (v, what) in [(c.width, "width"), (c.height, "height")] {
        match v {
            Some(Len::Px(p)) => e = if what == "width" { e.w(p) } else { e.h(p) },
            Some(Len::Pct(p)) if (p - 100.0).abs() < 1e-3 => e = if what == "width" { e.w_full() } else { e.h_full() },
            Some(l @ Len::Pct(_)) => note(what, &l, "only 100% sizes are mapped"),
            _ => {}
        }
    }
    for (v, f) in [(c.min_width, El::min_w as fn(El, f32) -> El), (c.min_height, El::min_h), (c.max_width, El::max_w), (c.max_height, El::max_h)] {
        match v {
            Some(Len::Px(p)) => e = f(e, p),
            Some(l @ Len::Pct(_)) => note("min/max size", &l, "only px min and max sizes are mapped"),
            _ => {}
        }
    }
    if let Some(g) = c.grow.filter(|g| *g > 0.0) {
        e = e.grow(g);
    } else if c.shrink.is_some_and(|s| s > 0.0) {
        e = e.shrink();
    }
    // Rune's grow(g) is `flex: g 1 0`, so a zero basis beside a grow is exact (flex-1); any other basis isn't
    let zero = |b: &Len| matches!(b, Len::Px(0.0) | Len::Pct(0.0));
    if let Some(b) = c.basis.filter(|b| *b != Len::Auto && !(zero(b) && c.grow.is_some_and(|g| g > 0.0))) {
        note("flexBasis", &b, "Rune sets no flex-basis of its own");
    }
    if let Some(a) = c.align_items {
        e = match a {
            Align::Start => e.items_start(),
            Align::End => e.items_end(),
            Align::Center => e.items_center(),
            _ => e.items_stretch(),
        };
    }
    if let Some(a) = c.align_self {
        e = match a {
            Align::Start => e.self_start(),
            Align::End => e.self_end(),
            Align::Center => e.self_center(),
            _ => e.self_stretch(),
        };
    }
    if let Some(j) = c.justify {
        e = match j {
            Align::Start | Align::Stretch => e.justify_start(),
            Align::End => e.justify_end(),
            Align::Center => e.justify_center(),
            Align::Between => e.justify_between(),
            Align::Around => e.justify_around(),
            Align::Evenly => e.justify_evenly(),
        };
    }
    if flex && (c.row_gap.is_some() || c.column_gap.is_some()) {
        e = e.gaps(c.row_gap.unwrap_or(0.0), c.column_gap.unwrap_or(0.0));
    }
    let (pad, bw) = padding(c);
    e = e.pt(pad[0]).pr(pad[1]).pb(pad[2]).pl(pad[3]);
    for (i, m) in c.margin.iter().enumerate() {
        match m {
            Some(Len::Px(p)) => e = [El::mt as fn(El, f32) -> El, El::mr, El::mb, El::ml][i](e, *p),
            Some(l) => note("margin", l, "auto and % margins aren't mapped"),
            None => {}
        }
    }
    if c.absolute {
        e = e.pin(c.inset[3], c.inset[0], c.inset[1], c.inset[2]);
    }
    e = paint(e, c, inh, bw, path, report);
    e
}

fn paint(mut e: El, c: &Css, inh: &Inherit, bw: [f32; 4], path: &str, report: &mut Report) -> El {
    if let Some(b) = c.background {
        e = e.fill(b);
    }
    if let Some((angle, stops)) = &c.gradient {
        e = e.linear_gradient(*angle, stops);
    }
    let radius = c.radius.map(|r| r.unwrap_or(0.0));
    if radius.iter().any(|r| *r > 0.0) {
        e = e.corners(radius);
    }
    let border = c.border_color.unwrap_or(inh.color);
    if bw.iter().all(|w| *w == bw[0]) {
        if bw[0] > 0.0 {
            e = e.border(bw[0], border);
        }
    } else {
        // one-sided borders: a line pinned along each side (Rune draws only a ring)
        if radius.iter().any(|r| *r > 0.0) {
            report.note(path, "border", "one-sided with a radius", "one-sided borders are drawn square");
        }
        let [t, r, b, l] = bw;
        let line = |e: El, w: f32, pin: (Option<f32>, Option<f32>, Option<f32>, Option<f32>), horizontal: bool| {
            if w <= 0.0 {
                return e;
            }
            let ln = el().fill(border).inert().pin(pin.0, pin.1, pin.2, pin.3);
            e.child(if horizontal { ln.h(w) } else { ln.w(w) })
        };
        e = line(e, t, (Some(0.0), Some(0.0), Some(0.0), None), true);
        e = line(e, b, (Some(0.0), None, Some(0.0), Some(0.0)), true);
        e = line(e, l, (Some(0.0), Some(0.0), None, Some(0.0)), false);
        e = line(e, r, (None, Some(0.0), Some(0.0), Some(0.0)), false);
    }
    for s in &c.shadows {
        e = e.box_shadow(Shadow { x: s.x, y: s.y, blur: s.blur, spread: s.spread, color: s.color, inset: s.inset });
    }
    if let Some(o) = c.opacity {
        e = e.opacity(o);
    }
    match (c.overflow_x, c.overflow_y) {
        (Overflow::Scroll, Overflow::Scroll) => e = e.scroll_xy(),
        (Overflow::Scroll, _) => e = e.scroll_x(),
        (_, Overflow::Scroll) => e = e.scroll(ScrollSpec { stick_bottom: false }),
        (Overflow::Hidden, _) | (_, Overflow::Hidden) => e = e.clip_corners(radius),
        _ => {}
    }
    e
}
