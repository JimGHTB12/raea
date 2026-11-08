//! raea: bevy_ui_widgets' Popover after layout (sdk/render's `popover`, `data-raea-bevy-popover`): the popup is moved
//! next to its parent's box, inside its border, at the first of its placements that the window (less its margin)
//! hides least, as bevy_ui_widgets' position_popover does after Bevy's layout. Its layout location is what moves, so
//! it's drawn and hit tested there, as Bevy's UiGlobalTransform is.

use crate::BaseDocument;
use markup5ever::LocalName;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Align {
    Start,
    End,
    Center,
}

/// `data-raea-bevy-popover="MARGIN;SIDE ALIGN GAP;..."` (Bottom Start 2): the window margin and the placements, in order
pub(crate) fn spec(v: &str) -> Option<(f32, Vec<(Side, Align, f32)>)> {
    let mut it = v.split(';');
    let margin = it.next()?.trim().parse().ok()?;
    let mut out = Vec::new();
    for p in it {
        let mut w = p.split_whitespace();
        let side = match w.next()? {
            "Top" => Side::Top,
            "Bottom" => Side::Bottom,
            "Left" => Side::Left,
            "Right" => Side::Right,
            _ => return None,
        };
        let align = match w.next()? {
            "Start" => Align::Start,
            "End" => Align::End,
            "Center" => Align::Center,
            _ => return None,
        };
        out.push((side, align, w.next()?.parse().ok()?));
    }
    Some((margin, out))
}

/// A rect as min x, min y, max x, max y
type Rect = [f32; 4];

/// The placement position_popover picks: the popup (`w` x `h`) beside `parent` at each placement, the one the window
/// rect hides least (the first of equals); its rect.
pub(crate) fn place(parent: Rect, (w, h): (f32, f32), window: Rect, at: &[(Side, Align, f32)]) -> Option<Rect> {
    let mut best: Option<(f32, Rect)> = None;
    for &(side, align, gap) in at {
        let mut r = [0.0f32; 4];
        match side {
            Side::Top => (r[3], r[1]) = (parent[1] - gap, parent[1] - gap - h),
            Side::Bottom => (r[1], r[3]) = (parent[3] + gap, parent[3] + gap + h),
            Side::Left => (r[2], r[0]) = (parent[0] - gap, parent[0] - gap - w),
            Side::Right => (r[0], r[2]) = (parent[2] + gap, parent[2] + gap + w),
        }
        let vertical = matches!(side, Side::Top | Side::Bottom);
        let (lo, hi, len, i) = if vertical { (parent[0], parent[2], w, 0) } else { (parent[1], parent[3], h, 1) };
        let start = match align {
            Align::Start => lo,
            Align::End => hi - len,
            Align::Center => lo + (hi - lo - len) * 0.5,
        };
        (r[i], r[i + 2]) = (start, start + len);
        let area = |q: Rect| (q[2] - q[0]).max(0.0) * (q[3] - q[1]).max(0.0);
        let clipped = [r[0].max(window[0]), r[1].max(window[1]), r[2].min(window[2]), r[3].min(window[3])];
        let hidden = area(r) - area(clipped);
        if best.is_none_or(|(b, _)| hidden < b) {
            best = Some((hidden, r));
        }
    }
    best.map(|(_, r)| r)
}

impl BaseDocument {
    pub(crate) fn resolve_bevy_popovers(&mut self) {
        let name = LocalName::from("data-raea-bevy-popover");
        let pops: Vec<(crate::NodeId, f32, Vec<(Side, Align, f32)>)> = self.nodes.iter().filter_map(|(id, n)| n.attr(name.clone()).and_then(spec).map(|(m, at)| (id, m, at))).collect();
        if pops.is_empty() {
            return;
        }
        let v = self.viewport();
        let scale = v.scale();
        let (vw, vh) = (v.window_size.0 as f32 / scale, v.window_size.1 as f32 / scale);
        for (id, margin, at) in pops {
            let Some(parent) = self.nodes[id].parent else { continue };
            let p = &self.nodes[parent];
            let (pl, b) = (p.final_layout(), p.final_layout().border);
            let o = p.absolute_position(0.0, 0.0);
            let rect = [o.x + b.left, o.y + b.top, o.x + pl.size.width - b.right, o.y + pl.size.height - b.bottom];
            let me = &self.nodes[id];
            let size = (me.final_layout().size.width, me.final_layout().size.height);
            let now = me.absolute_position(0.0, 0.0);
            let window = [margin, margin, vw - margin, vh - margin];
            let Some(r) = place(rect, size, window, &at) else { continue };
            let (dx, dy) = (r[0] - now.x, r[1] - now.y);
            if dx == 0.0 && dy == 0.0 {
                continue;
            }
            let node = &mut self.nodes[id];
            let l = node.final_layout_mut();
            (l.location.x, l.location.y) = (l.location.x + dx, l.location.y + dy);
            let l = node.unrounded_layout_mut();
            (l.location.x, l.location.y) = (l.location.x + dx, l.location.y + dy);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn below_at_the_start_unless_the_window_hides_more_of_it_than_above() {
        let at = spec("10;Bottom Start 2;Top Start 2").unwrap().1;
        // a 200 x 65 button at (540, 418) with a 5 px border: its inside 190 x 55, the popup 200 x 178 under it
        let parent = [545.0, 423.0, 735.0, 478.0];
        assert_eq!(place(parent, (200.0, 178.0), [10.0, 10.0, 1270.0, 710.0], &at), Some([545.0, 480.0, 745.0, 658.0]));
        // near the window's bottom: above
        let low = [545.0, 600.0, 735.0, 655.0];
        assert_eq!(place(low, (200.0, 178.0), [10.0, 10.0, 1270.0, 710.0], &at), Some([545.0, 420.0, 745.0, 598.0]));
        assert_eq!(spec("10;Middle Start 2"), None);
    }
}
