//! SVG image data and CSS intrinsic sizing for SVG.

use std::sync::Arc;

use usvg::roxmltree;

/// Dimensions declared on the root `<svg>` element, before any resolution.
///
/// Unlike [`usvg::Tree::size`], which always produces a concrete size, this
/// preserves what the SVG actually declared: absent attributes are `None` and
/// percentage lengths are kept unresolved.
#[derive(Debug, Clone, Copy, Default)]
pub struct SvgIntrinsicDimensions {
    /// The root `width` attribute, if declared. Percentages are unresolved.
    pub width: Option<svgtypes::Length>,
    /// The root `height` attribute, if declared. Percentages are unresolved.
    pub height: Option<svgtypes::Length>,
    /// The root `viewBox`, if declared and valid.
    pub view_box: Option<usvg::NonZeroRect>,
    /// The root `preserveAspectRatio` (default `xMidYMid meet`).
    pub preserve_aspect_ratio: svgtypes::AspectRatio,
    /// Whether the root declared a `viewBox` with a zero width or height,
    /// which disables rendering of the element per the SVG spec.
    pub degenerate_view_box: bool,
}

impl SvgIntrinsicDimensions {
    /// Extract the `width`/`height`/`viewBox` attributes declared on the root
    /// element of an already-parsed SVG document, with absent attributes as
    /// `None` and percentages unresolved. [`usvg::Tree`] does not preserve
    /// these (it always resolves to a concrete size), so they are read from
    /// the XML document here.
    pub fn from_xmltree(doc: &roxmltree::Document) -> Self {
        let root = doc.root_element();

        let parse_length = |name: &str| -> Option<svgtypes::Length> {
            root.attribute(name)?.parse::<svgtypes::Length>().ok()
        };
        // Parsed manually rather than via `svgtypes::ViewBox`, which rejects
        // zero sizes: a zero `viewBox` width/height is distinguished from an
        // invalid `viewBox` as it disables rendering of the element.
        let view_box_dims = root.attribute("viewBox").and_then(|s| {
            let mut numbers = svgtypes::NumberListParser::from(s);
            let x = numbers.next()?.ok()? as f32;
            let y = numbers.next()?.ok()? as f32;
            let w = numbers.next()?.ok()? as f32;
            let h = numbers.next()?.ok()? as f32;
            (numbers.next().is_none() && w.is_finite() && h.is_finite() && w >= 0.0 && h >= 0.0)
                .then_some((x, y, w, h))
        });
        let view_box =
            view_box_dims.and_then(|(x, y, w, h)| usvg::NonZeroRect::from_xywh(x, y, w, h));
        let degenerate_view_box = view_box_dims.is_some_and(|(_, _, w, h)| w == 0.0 || h == 0.0);
        let preserve_aspect_ratio = root
            .attribute("preserveAspectRatio")
            .and_then(|s| s.parse().ok())
            .unwrap_or_default();

        Self {
            width: parse_length("width"),
            height: parse_length("height"),
            view_box,
            preserve_aspect_ratio,
            degenerate_view_box,
        }
    }
}

/// A parsed SVG image.
///
/// usvg always resolves the root `<svg>` to a concrete [`usvg::Tree::size`],
/// falling back to the `viewBox` size when `width`/`height` are absent or given
/// as percentages. For CSS sizing purposes, however, such an SVG has *no*
/// intrinsic width/height (only an intrinsic aspect ratio). The accessors on
/// this type resolve the CSS intrinsic dimensions from the declared root
/// attributes, which are captured at parse time.
#[derive(Debug, Clone)]
pub struct SvgImageData {
    /// The parsed SVG tree.
    pub tree: Arc<usvg::Tree>,
    /// The dimensions declared on the root `<svg>` element.
    pub intrinsic_dimensions: SvgIntrinsicDimensions,
    /// raea: the source, for `tree_at`'s parse at a box's size
    source: Option<Arc<str>>,
    /// raea: `tree_at`'s last parse, by the size it was parsed at
    sized: Arc<std::sync::Mutex<Option<((u32, u32), Arc<usvg::Tree>)>>>,
}

impl SvgImageData {
    /// Parse an SVG image from raw data, capturing both the rendered
    /// [`usvg::Tree`] and the declared root dimensions from a single XML
    /// parse.
    ///
    /// Like [`usvg::Tree::from_data`], gzip-compressed data (SVGZ) is
    /// decompressed first.
    pub fn from_data(data: &[u8], options: &usvg::Options) -> Result<Self, usvg::Error> {
        // Gzip magic bytes, matching the SVGZ detection in `usvg::Tree::from_data`.
        let decompressed;
        let data = if data.starts_with(&[0x1f, 0x8b]) {
            decompressed = usvg::decompress_svgz(data)?;
            decompressed.as_slice()
        } else {
            data
        };

        let text = std::str::from_utf8(data).map_err(|_| usvg::Error::NotAnUtf8Str)?;
        // raea: elements with vector-effect="non-scaling-stroke" marked by id for anyrender_svg (usvg drops it)
        let marked = mark_non_scaling_strokes(text);
        let text = marked.as_deref().unwrap_or(text);
        let xml_options = roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        };
        let doc = roxmltree::Document::parse_with_options(text, xml_options)
            .map_err(usvg::Error::ParsingFailed)?;
        let tree = usvg::Tree::from_xmltree(&doc, options)?;
        Ok(Self {
            tree: Arc::new(tree),
            intrinsic_dimensions: SvgIntrinsicDimensions::from_xmltree(&doc),
            source: Some(Arc::from(text)),
            sized: Default::default(),
        })
    }

    /// raea: the tree as drawn in a box `width` x `height` CSS px. An SVG without a viewBox whose root `width` or
    /// `height` is missing or a percentage has the box as its viewport, so its percentages resolve against the box as
    /// Chrome's do; usvg took 100 x 100. Parsed again, with the box's size written on the root (usvg 0.48 reads the
    /// root's attributes again where it would use `Options::default_size`), only when the box's size changes. Drawn
    /// unscaled (blitz-paint's draw_svg).
    pub fn tree_at(&self, width: f32, height: f32) -> Arc<usvg::Tree> {
        use svgtypes::LengthUnit;
        let d = &self.intrinsic_dimensions;
        let sized_by_box = |l: Option<svgtypes::Length>| l.is_none_or(|l| l.unit == LengthUnit::Percent);
        let Some(source) = &self.source else {
            return self.tree.clone();
        };
        if d.view_box.is_some() || !(sized_by_box(d.width) || sized_by_box(d.height)) || width <= 0.0 || height <= 0.0 {
            return self.tree.clone();
        }
        let key = (width.round() as u32, height.round() as u32);
        let mut sized = self.sized.lock().unwrap();
        if let Some((_, t)) = sized.as_ref().filter(|(k, _)| *k == key) {
            return t.clone();
        }
        let options = usvg::Options { fontdb: crate::util::svg_fonts(), ..Default::default() };
        let tree = sized_root(source, sized_by_box(d.width).then_some(width), sized_by_box(d.height).then_some(height))
            .and_then(|text| {
                let xml_options = roxmltree::ParsingOptions { allow_dtd: true, ..Default::default() };
                let doc = roxmltree::Document::parse_with_options(&text, xml_options).ok()?;
                usvg::Tree::from_xmltree(&doc, &options).ok()
            })
            .map(Arc::new)
            .unwrap_or_else(|| self.tree.clone());
        *sized = Some((key, tree.clone()));
        tree
    }

    /// The intrinsic width in CSS px, present only when the root `<svg>`
    /// declared an absolute (non-percentage) `width`.
    pub fn intrinsic_width(&self) -> Option<f32> {
        use svgtypes::LengthUnit;
        let declared = self
            .intrinsic_dimensions
            .width
            .is_some_and(|len| len.unit != LengthUnit::Percent);
        declared.then(|| self.tree.size().width())
    }

    /// The intrinsic height in CSS px, present only when the root `<svg>`
    /// declared an absolute (non-percentage) `height`.
    pub fn intrinsic_height(&self) -> Option<f32> {
        use svgtypes::LengthUnit;
        let declared = self
            .intrinsic_dimensions
            .height
            .is_some_and(|len| len.unit != LengthUnit::Percent);
        declared.then(|| self.tree.size().height())
    }

    /// The aspect ratio of the root `<svg>`'s `viewBox`, if it declares one.
    pub fn viewbox_aspect_ratio(&self) -> Option<f32> {
        self.intrinsic_dimensions
            .view_box
            .map(|view_box| view_box.width() / view_box.height())
    }

    /// The transform from the rendered tree (in [`usvg::Tree::size`] units) to a
    /// viewport of the given size, applying the root `viewBox` and
    /// `preserveAspectRatio`. Without a `viewBox` the tree is stretched to fill it.
    pub fn viewport_transform(&self, width: f32, height: f32) -> usvg::Transform {
        let size = self.tree.size();
        let stretch = usvg::Transform::from_scale(width / size.width(), height / size.height());
        let Some(view_box) = self.intrinsic_dimensions.view_box else {
            return stretch;
        };
        let aspect = self.intrinsic_dimensions.preserve_aspect_ratio;
        // usvg has already mapped the `viewBox` onto `Tree::size`, so undo that first.
        let view_box_to_tree = view_box_transform(view_box, aspect, size.width(), size.height());
        match view_box_to_tree.invert() {
            Some(tree_to_view_box) => {
                view_box_transform(view_box, aspect, width, height).pre_concat(tree_to_view_box)
            }
            None => stretch,
        }
    }

    /// The root `width` attribute resolved against a containing block width:
    /// percentages resolve against the containing block (`None` if it is
    /// indefinite) and an absent attribute is `None`.
    ///
    /// This is only appropriate for an inline `<svg>` element, where the
    /// attributes behave as presentation attributes. SVG used as an image
    /// (e.g. `<img src>` or a background) must use [`Self::intrinsic_width`],
    /// as its intrinsic dimensions are context-free per CSS.
    pub fn resolved_width(&self, container_width: Option<f32>) -> Option<f32> {
        use svgtypes::LengthUnit;
        match self.intrinsic_dimensions.width {
            Some(len) if len.unit != LengthUnit::Percent => Some(self.tree.size().width()),
            Some(len) => container_width.map(|cw| cw * (len.number as f32) / 100.0),
            None => None,
        }
    }

    /// The root `height` attribute resolved against a containing block height.
    /// See [`Self::resolved_width`].
    pub fn resolved_height(&self, container_height: Option<f32>) -> Option<f32> {
        use svgtypes::LengthUnit;
        match self.intrinsic_dimensions.height {
            Some(len) if len.unit != LengthUnit::Percent => Some(self.tree.size().height()),
            Some(len) => container_height.map(|ch| ch * (len.number as f32) / 100.0),
            None => None,
        }
    }

    /// The intrinsic aspect ratio of the SVG: the ratio of its declared
    /// `width`/`height` when both are absolute lengths, otherwise the
    /// `viewBox` ratio, otherwise the ratio of the resolved
    /// [`usvg::Tree::size`] (which is always non-zero).
    pub fn aspect_ratio(&self) -> f32 {
        match (self.intrinsic_width(), self.intrinsic_height()) {
            (Some(w), Some(h)) => w / h,
            _ => self.viewbox_aspect_ratio().unwrap_or_else(|| {
                let size = self.tree.size();
                size.width() / size.height()
            }),
        }
    }

    /// The intrinsic dimensions of the SVG resolved per CSS replaced element
    /// sizing: a missing dimension is computed from the declared one and the
    /// intrinsic aspect ratio; if neither is declared, the resolved
    /// [`usvg::Tree::size`] is used as a fallback.
    pub fn intrinsic_size(&self) -> (f32, f32) {
        let aspect_ratio = self.aspect_ratio();
        match (self.intrinsic_width(), self.intrinsic_height()) {
            (Some(w), Some(h)) => (w, h),
            (Some(w), None) => (w, w / aspect_ratio),
            (None, Some(h)) => (h * aspect_ratio, h),
            (None, None) => {
                // No intrinsic dimensions. If there is an intrinsic aspect ratio, apply
                // the CSS default sizing algorithm: contain within the default object
                // size of 300x150. Otherwise fall back to the resolved tree size.
                if self.viewbox_aspect_ratio().is_some() {
                    let scale = (300.0 / aspect_ratio).min(150.0);
                    (scale * aspect_ratio, scale)
                } else {
                    let size = self.tree.size();
                    (size.width(), size.height())
                }
            }
        }
    }
}

/// Map a `viewBox` onto a viewport of the given size per `preserveAspectRatio`.
/// raea: CSS `filter` with url()s on an HTML element (docs/blitz-bevy.feature B4a): the element's filter list,
/// applied by usvg to a rect the size of its border box (`width` x `height` CSS px, its top left at 0, 0), with the
/// `<filter>`s its url()s name in the page serialized beside it. usvg reads the filter elements' primitives and the
/// list's functions as it does an SVG's, so anyrender_svg's graph for SVG draws it; with the ids of the `<filter>`s.
/// None when a url() names nothing or something that isn't a `<filter>`: the list is ignored then, as in Chrome.
pub fn css_filter_tree(doc: &crate::BaseDocument, list: &[style::computed_values::filter::single_value::T], width: f32, height: f32) -> Option<(usvg::Tree, Vec<String>)> {
    use style::computed_values::filter::single_value::T as Filter;
    use style_traits::ToCss;
    let (mut css, mut defs, mut ids) = (vec![], String::new(), vec![]);
    for f in list {
        let text = f.to_css_string();
        if let Filter::Url(_) = f {
            // url("…#id") as the page wrote it, or resolved against its base: the fragment is the element
            let (id, node) = url_element(doc, &text, "filter")?;
            defs.push_str(&node.svg_source());
            css.push(format!("url(#{id})"));
            ids.push(id.to_string());
        } else {
            css.push(text.replace('"', "'"));
        }
    }
    let list = css.join(" ");
    Some((rect_tree(&defs, &format!(r#"filter="{list}""#), width, height)?, ids))
}

/// raea: `clip-path: url(#id)` on an HTML element: the `<clipPath>` it names, serialized beside a rect the size of
/// the element's border box (`width` x `height` CSS px) that it clips, so usvg resolves its units (userSpaceOnUse
/// from the box's top left, objectBoundingBox the box), transforms and clip-rule as an SVG's (anyrender_svg's
/// clip_path_shape reads them back). None when the url names nothing or no `<clipPath>`: no clip then, as in Chrome.
pub fn css_clip_tree(doc: &crate::BaseDocument, url: &style::values::computed::url::ComputedUrl, width: f32, height: f32) -> Option<usvg::Tree> {
    use style_traits::ToCss;
    let url = url.to_css_string();
    let (id, node) = url_element(doc, &url, "clipPath")?;
    rect_tree(&node.svg_source(), &format!(r#"clip-path="url(#{id})""#), width, height)
}

/// The element a CSS url("…#id") names (as the page wrote it, or resolved against its base: the fragment is the
/// id), when it's a `name` element; with its id.
fn url_element<'a>(doc: &'a crate::BaseDocument, url: &'a str, name: &str) -> Option<(&'a str, &'a crate::Node)> {
    let id = url.rsplit_once('#')?.1.trim_end_matches([')', '"', '\'']);
    let node = doc.get_node(doc.get_element_by_id(id)?)?;
    (node.element_data().map(|e| &*e.name.local) == Some(name)).then_some((id, node))
}

/// An SVG `width` x `height` with `defs` and one rect that size with the attributes `attrs`, as usvg reads it.
fn rect_tree(defs: &str, attrs: &str, width: f32, height: f32) -> Option<usvg::Tree> {
    let src = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{width}" height="{height}"><defs>{defs}</defs><rect width="{width}" height="{height}" {attrs}/></svg>"#
    );
    let xml = roxmltree::Document::parse_with_options(&src, roxmltree::ParsingOptions { allow_dtd: true, ..Default::default() }).ok()?;
    usvg::Tree::from_xmltree(&xml, &usvg::Options::default()).ok()
}

fn view_box_transform(
    view_box: usvg::NonZeroRect,
    aspect: svgtypes::AspectRatio,
    width: f32,
    height: f32,
) -> usvg::Transform {
    use svgtypes::Align;

    let sx = width / view_box.width();
    let sy = height / view_box.height();
    let (sx, sy) = if aspect.align == Align::None {
        (sx, sy)
    } else if aspect.slice {
        (sx.max(sy), sx.max(sy))
    } else {
        (sx.min(sy), sx.min(sy))
    };
    let (align_x, align_y) = match aspect.align {
        Align::None | Align::XMinYMin => (0.0, 0.0),
        Align::XMidYMin => (0.5, 0.0),
        Align::XMaxYMin => (1.0, 0.0),
        Align::XMinYMid => (0.0, 0.5),
        Align::XMidYMid => (0.5, 0.5),
        Align::XMaxYMid => (1.0, 0.5),
        Align::XMinYMax => (0.0, 1.0),
        Align::XMidYMax => (0.5, 1.0),
        Align::XMaxYMax => (1.0, 1.0),
    };
    let tx = (width - view_box.width() * sx) * align_x - view_box.x() * sx;
    let ty = (height - view_box.height() * sy) * align_y - view_box.y() * sy;
    usvg::Transform::from_row(sx, 0.0, 0.0, sy, tx, ty)
}

/// raea: the id prefix that marks an element whose stroke is drawn with vector-effect="non-scaling-stroke"
/// (anyrender_svg's render.rs strokes such a path in its SVG's viewport space, where Chrome does). usvg 0.48 parses
/// vector-effect and drops it, but keeps a path's id.
pub const NON_SCALING_STROKE_ID: &str = "raea-nss-";

/// raea: `source` with each element that says vector-effect="non-scaling-stroke" (as an attribute or in its style)
/// given an id starting NON_SCALING_STROKE_ID: a new one, or its own renamed when nothing refers to it. None when
/// there's none (or it doesn't parse).
fn mark_non_scaling_strokes(source: &str) -> Option<String> {
    if !source.contains("non-scaling-stroke") {
        return None;
    }
    let opts = roxmltree::ParsingOptions { allow_dtd: true, ..Default::default() };
    let doc = roxmltree::Document::parse_with_options(source, opts).ok()?;
    let says = |n: &roxmltree::Node| {
        n.attribute("vector-effect").is_some_and(|v| v.trim() == "non-scaling-stroke")
            || n.attribute("style").is_some_and(|st| {
                st.split(';').any(|d| d.split_once(':').is_some_and(|(k, v)| k.trim() == "vector-effect" && v.trim() == "non-scaling-stroke"))
            })
    };
    // an id of its own is renamed to carry the mark when nothing in the source refers to it (`#id`): WPT's
    // svgbox-stroke-box-003 strokes `<rect id="target">` (raea #50); one referred to keeps its id, and its stroke scales
    let mut edits: Vec<(std::ops::Range<usize>, Option<String>)> = doc
        .descendants()
        .filter(|n| n.is_element() && says(n))
        .filter_map(|n| match n.attributes().find(|a| a.name() == "id") {
            Some(a) if !source.contains(&format!("#{}", a.value())) => Some((a.range(), Some(a.value().to_string()))),
            Some(_) => None,
            // after the start tag's name
            None => {
                let start = n.range().start + 1;
                let end = start + source[start..].find(|c: char| c.is_whitespace() || c == '/' || c == '>')?;
                Some((end..end, None))
            }
        })
        .collect();
    if edits.is_empty() {
        return None;
    }
    // numbered in source order, applied from the last so the earlier offsets hold
    edits.sort_unstable_by_key(|(r, _)| r.start);
    let mut out = source.to_string();
    for (k, (r, own)) in edits.into_iter().enumerate().rev() {
        let id = match own {
            Some(own) => format!("id=\"{NON_SCALING_STROKE_ID}{k}-{own}\""),
            None => format!(" id=\"{NON_SCALING_STROKE_ID}{k}\""),
        };
        out.replace_range(r, &id);
    }
    Some(out)
}

/// raea: `source` with its root `<svg>`'s `width` and `height` set to these (each one given replaces the attribute,
/// whatever it was, or is added). None when the root's start tag isn't found.
fn sized_root(source: &str, width: Option<f32>, height: Option<f32>) -> Option<String> {
    let start = source.find("<svg")?;
    // the start tag's end: the first `>` outside a quoted value
    let mut quote = None;
    let end = start + source[start..].char_indices().find(|&(_, c)| match quote {
        Some(q) => {
            if c == q {
                quote = None;
            }
            false
        }
        None if c == '"' || c == '\'' => {
            quote = Some(c);
            false
        }
        None => c == '>',
    })?.0;
    let mut tag = source[start..end].to_string();
    for (name, value) in [("width", width), ("height", height)] {
        let Some(value) = value else { continue };
        // the attribute, if present: whitespace, the name, `=`, a quoted value
        let at = tag.char_indices().filter(|&(_, c)| c.is_whitespace()).map(|(i, _)| i).find(|&i| {
            let rest = tag[i..].trim_start();
            rest.strip_prefix(name).is_some_and(|r| r.trim_start().starts_with('='))
        });
        if let Some(at) = at {
            let eq = at + tag[at..].find('=')?;
            let open = eq + tag[eq..].find(['"', '\''])?;
            let q = tag[open..].chars().next()?;
            let close = open + 1 + tag[open + 1..].find(q)?;
            tag.replace_range(at..close + 1, "");
        }
        let at = tag.trim_end_matches('/').len();
        tag.insert_str(at, &format!(" {name}=\"{value}\""));
    }
    Some(format!("{}{tag}{}", &source[..start], &source[end..]))
}

#[cfg(test)]
mod non_scaling_stroke_tests {
    /// raea: a path with vector-effect="non-scaling-stroke" (attribute or style) keeps blitz-dom's mark through usvg
    #[test]
    fn marked_paths_keep_their_mark() {
        let src = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 24"><polyline points="0,0 48,24" stroke="red" vector-effect="non-scaling-stroke"/><rect width="4" height="4" stroke="red" style="vector-effect: non-scaling-stroke"/><path id="own" d="M0 0L1 1" stroke="red" vector-effect="non-scaling-stroke"/><path d="M0 0L2 2" stroke="red"/><path id="kept" d="M0 0L3 3" stroke="red" vector-effect="non-scaling-stroke"/><use href="#kept"/></svg>"##;
        let svg = super::SvgImageData::from_data(src.as_bytes(), &usvg::Options::default()).unwrap();
        let ids: Vec<String> = svg.tree.root().children().iter().filter_map(|n| match n {
            usvg::Node::Path(p) => Some(p.id().to_string()),
            _ => None,
        }).collect();
        // an id of its own carries the mark when nothing refers to it (#50); `kept` is used, so it stays
        assert_eq!(ids, ["raea-nss-0", "raea-nss-1", "raea-nss-2-own", "", "kept"]);
    }
}
