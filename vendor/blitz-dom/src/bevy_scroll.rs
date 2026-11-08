//! raea: bevy_ui's scrolling after layout. A scroll offset the page set (scrollTop, scrollLeft) is kept in its
//! scroller's range, as Chrome clamps scrollTop, and a bevy_ui scroller's is floored to whole physical px, as bevy_ui's
//! layout floors ScrollPosition. A bevy_ui_widgets Scrollbar's thumb is sized and placed in its track from its
//! target's offset, as bevy_ui_widgets' update_scrollbar_thumb does (sdk/render's Scrollbar and ScrollbarThumb).

use crate::{BaseDocument, NodeId};
use markup5ever::LocalName;

/// A Scrollbar track's `data-raea-bevy-scrollbar`: "v" or "h", its min_thumb_length (px) and its target's HTML id
struct Bar {
    vertical: bool,
    min: f32,
    target: String,
}

fn bar(v: &str) -> Option<Bar> {
    let mut it = v.splitn(3, ' ');
    let vertical = match it.next()? {
        "v" => true,
        "h" => false,
        _ => return None,
    };
    let min = it.next()?.parse().ok()?;
    Some(Bar { vertical, min, target: it.next()?.to_string() })
}

/// bevy_ui_widgets' size_and_pos: the thumb's length and its offset along the track
pub fn size_and_pos(content: f32, visible: f32, track: f32, min: f32, offset: f32) -> (f32, f32) {
    if content <= visible {
        return (track, 0.0);
    }
    let size = (track * visible / content).max(min).min(track);
    let offset = offset.clamp(0.0, content - visible);
    (size, offset * (track - size) / (content - visible))
}

impl BaseDocument {
    pub(crate) fn resolve_bevy_scroll(&mut self) {
        self.scroll_set.retain(|id| self.nodes.get(*id).is_some());
        for &id in &self.scroll_set {
            let node = &mut self.nodes[id];
            let (max_x, max_y) = (node.final_layout().scroll_width() as f64, node.final_layout().scroll_height() as f64);
            let k = node.bevy_scroll_scale();
            let at = node.scroll_offset_mut();
            for (v, max) in [(&mut at.x, max_x), (&mut at.y, max_y)] {
                *v = v.clamp(0.0, max);
                if let Some(k) = k {
                    *v = (*v * k).floor() / k;
                }
            }
        }
        self.scrollbars.retain(|id| self.nodes.get(*id).is_some());
        for i in 0..self.scrollbars.len() {
            self.place_thumbs(self.scrollbars[i]);
        }
    }

    fn place_thumbs(&mut self, track: NodeId) {
        let Some(b) = self.nodes[track].attr(LocalName::from("data-raea-bevy-scrollbar")).and_then(bar) else { return };
        let Some(target) = self.get_element_by_id(&b.target) else { return };
        let t = &self.nodes[target];
        let l = t.final_layout();
        let visible = (l.size.width - l.scrollbar_size.width, l.size.height - l.scrollbar_size.height);
        let content = (l.scrollable_overflow_rect.right, l.scrollable_overflow_rect.bottom);
        // (the offset the page asked for: bevy_ui_widgets reads ScrollPosition, which layout never clamps)
        let asked = t.bevy_scroll_asked().unwrap_or((0.0, 0.0));
        let size = self.nodes[track].final_layout().size;
        let (len, pos) = if b.vertical {
            size_and_pos(content.1, visible.1, size.height, b.min, asked.1)
        } else {
            size_and_pos(content.0, visible.0, size.width, b.min, asked.0)
        };
        let thumbs: Vec<NodeId> = self.nodes[track].children.iter().copied().filter(|c| self.nodes[*c].attr(LocalName::from("data-raea-bevy-thumb")).is_some()).collect();
        let set = |l: &mut taffy::Layout| {
            if b.vertical {
                l.size = taffy::Size { width: size.width, height: len };
                l.location = taffy::Point { x: 0.0, y: pos };
            } else {
                l.size = taffy::Size { width: len, height: size.height };
                l.location = taffy::Point { x: pos, y: 0.0 };
            }
        };
        for c in thumbs {
            set(self.nodes[c].final_layout_mut());
            set(self.nodes[c].unrounded_layout_mut());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{bar, size_and_pos};

    #[test]
    fn thumb_is_the_share_in_view_at_the_offset() {
        // 300 of content, 100 in view, a 100 track: a third of it, 10 down moves it 10 * 66.7 / 200
        let (len, pos) = size_and_pos(300.0, 100.0, 100.0, 8.0, 10.0);
        assert!((len - 100.0 / 3.0).abs() < 1e-4);
        assert!((pos - 10.0 * (100.0 - len) / 200.0).abs() < 1e-4);
        // the least length holds, the offset is kept in range, and content that fits fills the track
        assert_eq!(size_and_pos(10_000.0, 100.0, 100.0, 8.0, 0.0).0, 8.0);
        assert_eq!(size_and_pos(300.0, 100.0, 100.0, 8.0, 900.0).1, 100.0 - len);
        assert_eq!(size_and_pos(50.0, 100.0, 100.0, 8.0, 40.0), (100.0, 0.0));
    }

    #[test]
    fn track_attr_is_orientation_length_and_target() {
        let b = bar("v 8 scroll-area").unwrap();
        assert!(b.vertical && b.min == 8.0 && b.target == "scroll-area");
        assert!(!bar("h 0 a").unwrap().vertical);
        assert!(bar("x 8 a").is_none() && bar("v eight a").is_none() && bar("v 8").is_none());
    }
}
