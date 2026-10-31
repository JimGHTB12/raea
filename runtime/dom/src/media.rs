//! The host page's `video` and `audio` (raea-runtime-js's media_html.rs, on the Blitz page): a source is decoded by raea-media
//! on a thread of its own (raea-runtime-js's stream), what it tells goes to the element's JS as raea-runtime-js's `media` host event
//! (`loadedmetadata`, `loadeddata`, `seeked`, `error`: sdk/ui's dom layer fires each at the element), and a video's
//! frame is its picture, which its natural size is. Its poster shows until the first seek; with none, the frame does.
//! A seek is the prop `__seek` ([time, n], set by the dom layer's `currentTime`).

use std::collections::HashMap;
use std::sync::Arc;

use blitz_dom::BaseDocument;
use blitz_dom::node::{ImageData, RasterImageData, SpecialElementData};
use raea_runtime_js::media_html::stream::{Msg, Stream};
use serde_json::{Value, json};

use raea_sys_page::ops::Nodes;

/// One media element's source and what its decoder has told.
pub struct Media {
    src: String,
    poster: Option<String>,
    seek: Option<Value>,
    stream: Option<Stream>,
    video: bool,
    loaded: bool,
    /// a seek asked and not landed yet
    seeking: Option<f64>,
    /// a seek has landed (the poster's gone)
    seeked: bool,
    duration: f64,
    /// the latest frame: width, height, pixels
    frame: Option<(u32, u32, Arc<Vec<u8>>)>,
}

/// Each media element by its wire id.
pub type Medias = HashMap<u64, Media>;

/// The elements' sources and seeks brought up to their props, and what their decoders told since: the `media` events
/// for the element's JS in `out`, and each video's picture set.
/// `draw`: a video's frames are its picture; false when the page's caller places them itself (html.rs, each at the
/// time a seek asks, not when a stream gets there), and only the events go to the JS.
pub fn sync(doc: &mut BaseDocument, nodes: &Nodes, medias: &mut Medias, out: &mut Vec<Value>, draw: bool) {
    medias.retain(|id, _| nodes.by_id.contains_key(id));
    for id in nodes.media_ids() {
        let video = nodes.kind(id) == Some("video");
        let props = nodes.props.get(&id);
        let s = |k: &str| props.and_then(|p| p.get(k)).and_then(Value::as_str).filter(|v| !v.is_empty()).map(String::from);
        let (src, poster, seek) = (s("src").unwrap_or_default(), s("poster"), props.and_then(|p| p.get("__seek")).cloned());
        let m = match medias.get_mut(&id) {
            Some(m) if m.src == src => m,
            _ => {
                let stream = (!src.is_empty()).then(|| raea_runtime_js::media_html::stream::open(&raea_sys_page::net::page_path(&src), video)).flatten();
                medias.insert(id, Media { src, poster: None, seek: None, stream, video, loaded: false, seeking: None, seeked: false, duration: f64::NAN, frame: None });
                medias.get_mut(&id).unwrap()
            }
        };
        let mut redraw = false;
        if m.poster != poster {
            m.poster = poster;
            redraw = true;
        }
        if m.seek != seek {
            if let Some(t) = seek.as_ref().and_then(|v| v.get(0)).and_then(Value::as_f64)
                && let Some(st) = m.stream.as_mut()
            {
                m.seeking = Some(t);
                st.seek(t);
            }
            m.seek = seek;
        }
        while let Some(msg) = m.stream.as_ref().and_then(Stream::next) {
            match msg {
                Msg::Meta { w, h, duration } => {
                    m.duration = duration;
                    out.push(if m.video { json!({"id": id, "type": "loadedmetadata", "w": w, "h": h, "duration": duration}) } else { json!({"id": id, "type": "loadedmetadata", "duration": duration}) });
                }
                Msg::Fail(err) => {
                    bevy::log::warn!("raea-page: media: {err}");
                    out.push(json!({"id": id, "type": "error", "message": err}));
                }
                Msg::Frame(t, f) => {
                    let (w, h) = (f.width, f.height);
                    m.frame = Some((w, h, Arc::new(f.rgba)));
                    redraw = true;
                    if !m.loaded {
                        m.loaded = true;
                        out.push(json!({"id": id, "type": "loadeddata", "w": w, "h": h, "duration": m.duration}));
                    }
                    if m.seeking.take_if(|s| (*s - t).abs() < 1e-9).is_some() {
                        m.seeked = true;
                        out.push(json!({"id": id, "type": "seeked", "time": t}));
                    }
                }
            }
        }
        if redraw && m.video && draw {
            let pic = match (&m.poster, m.seeked) {
                (Some(p), false) => poster_image(p),
                _ => None,
            }
            .or_else(|| m.frame.as_ref().map(|(w, h, px)| (*w, *h, px.as_ref().clone())));
            if let (Some((w, h, px)), Some(n)) = (pic, nodes.by_id.get(&id)) {
                set_image(doc, *n, w, h, px);
            }
        }
    }
}

/// A poster's picture, decoded as an img's source is.
fn poster_image(src: &str) -> Option<(u32, u32, Vec<u8>)> {
    let (mime, bytes) = match raea_io_net::data_url(src) {
        Some(d) => d,
        None => {
            let path = raea_sys_page::net::page_path(src);
            let ext = path.rsplit_once('.')?.1.to_ascii_lowercase();
            (format!("image/{ext}"), std::fs::read(&path).ok()?)
        }
    };
    let img = raea_sys_image::decode::decode(&bytes, &mime)?;
    let size = img.size();
    Some((size.x, size.y, img.data?))
}

/// Node `n` drawn as an image: `rgba` as its raster, its layout made again for the picture's size.
fn set_image(doc: &mut BaseDocument, n: blitz_dom::NodeId, w: u32, h: u32, rgba: Vec<u8>) {
    let Some(node) = doc.get_node_mut(n) else { return };
    let Some(el) = node.element_data_mut() else { return };
    el.special_data = SpecialElementData::Image(Box::new(ImageData::Raster(RasterImageData { width: w, height: h, data: vello::peniko::Blob::new(Arc::new(rgba)) })));
    node.clear_layout_cache();
    node.insert_damage(raea_sys_page::ALL_DAMAGE);
    node.mark_ancestors_dirty();
}
