use anyrender::PaintScene;
use blitz_dom::node::SpecialElementData;
use kurbo::{
    BezPath, Cap, Circle, Insets, Join, ParamCurveArclen as _, PathEl, PathSeg, Point, Rect, Shape as _,
    Stroke, Vec2,
};
use peniko::{Color, Fill};
use smallvec::SmallVec;
use style::{
    computed_values::border_collapse::T as BorderCollapse,
    values::computed::{BorderStyle, OutlineStyle},
};

use crate::{
    color::{ToColorColor as _, contrast_ratio},
    kurbo_css::Edge,
    render::{ElementCx, PhysicalTracks},
};

/// A darker version of a colour (mirrors WebKit/Blink's `Color::Dark`): the
/// brightest channel is reduced by a fixed amount and the others scaled to match,
/// preserving hue.
fn darken(color: Color) -> Color {
    let [r, g, b, a] = color.components;
    let v = r.max(g).max(b);
    if v == 0.0 {
        return color;
    }
    let multiplier = ((v - 0.33) / v).max(0.0);
    Color::new([r * multiplier, g * multiplier, b * multiplier, a])
}

/// A lighter version of a colour (mirrors WebKit/Blink's `Color::Light`): the
/// brightest channel is increased by a fixed amount and the others scaled to
/// match, preserving hue.
fn lighten(color: Color) -> Color {
    let [r, g, b, a] = color.components;
    let v = r.max(g).max(b);
    if v == 0.0 {
        // Pure black: WebKit uses a fixed dark grey (0x545454).
        return Color::new([0.33, 0.33, 0.33, a]);
    }
    let multiplier = (v + 0.33).min(1.0) / v;
    Color::new([r * multiplier, g * multiplier, b * multiplier, a])
}

/// The colour of a single edge of an `inset`/`outset` border.
///
/// An `outset` border is raised: the top/left edges use the "lighter" shade and
/// the bottom/right edges the "darker" shade (`inset` is the reverse). The exact
/// shading matches Chrome's `CalculateInsetOutsetColor`:
///
/// * The darker edge is always darkened.
/// * The lighter edge keeps the base colour unchanged, *except* for colours dark
///   enough that the darkened edge wouldn't have enough contrast against the base
///   colour, which are lightened instead so the bevel stays visible.
fn beveled_edge_color(color: Color, edge: Edge, inset: bool) -> Color {
    let top_or_left = matches!(edge, Edge::Top | Edge::Left);
    let should_darken = top_or_left == inset;

    let dark_color = darken(color);
    if should_darken {
        return dark_color;
    }

    // Lighter edge. Chrome uses the base colour as-is unless the colour is dark
    // enough that the darkened edge lacks contrast against it. The red/green
    // shortcut mirrors Chrome: for those colours the contrast is always
    // sufficient, so the base colour is used without a contrast check.
    let [r, g, _, _] = color.components;
    if r >= 150.0 / 255.0 || g >= 92.0 / 255.0 {
        return color;
    }

    // Lighten only when the darkened edge would be too low-contrast to read.
    const MIN_CONTRAST_RATIO: f32 = 1.75;
    if contrast_ratio(color, dark_color) < MIN_CONTRAST_RATIO {
        lighten(color)
    } else {
        color
    }
}

/// The (outer half, inner half) colours of a single edge of a `groove`/`ridge`
/// border.
///
/// A `groove` looks carved into the page: its outer half is shaded like `inset`
/// and its inner half like `outset`. `ridge` is the reverse (raised).
fn grooved_edge_colors(color: Color, edge: Edge, ridge: bool) -> (Color, Color) {
    let outer = beveled_edge_color(color, edge, !ridge);
    let inner = beveled_edge_color(color, edge, ridge);
    (outer, inner)
}

/// The `(dash, gap)` lengths of a dashed border, as multiples of the border
/// thickness.
///
/// Matches Chrome: borders that are at least 3px use a 2:1 dash-to-gap ratio,
/// while thinner borders use longer dashes and gaps (3:2) so they still read as
/// dashes rather than dots or a solid line. `thickness` and `scale` are in device
/// pixels; the 3px threshold is applied in CSS pixels.
fn dashed_ratios(thickness: f64, scale: f64) -> (f64, f64) {
    if thickness >= 3.0 * scale {
        (2.0, 1.0)
    } else {
        (3.0, 2.0)
    }
}

/// raea: Blink's SelectBestDashGap (styled_stroke_data.cc): keeping the dash
/// length, the gap that fits a whole number of dashes in `length` and is closest
/// to `gap`. An open line starts and ends on a dash; a closed path has as many
/// gaps as dashes.
fn best_dash_gap(length: f64, dash: f64, gap: f64, closed: bool) -> f64 {
    let available = if closed { length } else { length + gap };
    let min_dashes = (available / (dash + gap)).floor();
    let max_dashes = min_dashes + 1.0;
    let (min_gaps, max_gaps) = if closed {
        (min_dashes, max_dashes)
    } else {
        (min_dashes - 1.0, max_dashes - 1.0)
    };
    let min_gap = (length - min_dashes * dash) / min_gaps;
    let max_gap = (length - max_dashes * dash) / max_gaps;
    if max_gap <= 0.0 || (min_gap - gap).abs() < (max_gap - gap).abs() {
        min_gap
    } else {
        max_gap
    }
}

/// raea: the `(dash, gap)` Blink strokes a dashed (or thin dotted) line of
/// `length` with (DashEffectFromStrokeStyle); `None` draws it solid. Only dashed
/// lines (`fit`) have their gap fitted to the length.
fn blink_dashes(length: f64, dash: f64, gap: f64, fit: bool, closed: bool) -> Option<(f64, f64)> {
    if length <= dash * 2.0 {
        return None;
    }
    let two = 2.0 * dash + gap + if closed { gap } else { 0.0 };
    if length <= two {
        let m = length / two;
        return Some((dash * m, gap * m));
    }
    Some((dash, if fit { best_dash_gap(length, dash, gap, closed) } else { gap }))
}

/// raea: the dash offset that puts the start of a dash `start` along the path.
fn dash_phase(start: f64, period: f64) -> f64 {
    (period - start % period) % period
}

/// Return `count` points spaced `spacing` apart (by arc length) along the closed
/// `path`, the first `start` along it. Used to place dots along a rounded border.
fn sample_points_along_path(path: &BezPath, start: f64, spacing: f64, count: usize) -> Vec<Point> {
    // Flatten to a polyline so we can walk it by arc length.
    let mut poly: Vec<Point> = Vec::new();
    kurbo::flatten(path.iter(), 0.1, |el| match el {
        PathEl::MoveTo(p) | PathEl::LineTo(p) => poly.push(p),
        PathEl::ClosePath => {
            if let Some(&first) = poly.first() {
                poly.push(first);
            }
        }
        _ => {}
    });

    let mut points = Vec::with_capacity(count);
    if poly.len() < 2 {
        return points;
    }
    // Arc length at each polyline point.
    let mut along = vec![0.0];
    for w in poly.windows(2) {
        along.push(along.last().unwrap() + (w[1] - w[0]).hypot());
    }
    let total = *along.last().unwrap();
    if total <= 0.0 {
        return points;
    }

    for i in 0..count {
        let target = (start + i as f64 * spacing).rem_euclid(total);
        let seg = along.partition_point(|&a| a <= target).clamp(1, poly.len() - 1) - 1;
        let seg_len = along[seg + 1] - along[seg];
        let t = if seg_len > 0.0 {
            ((target - along[seg]) / seg_len).clamp(0.0, 1.0)
        } else {
            0.0
        };
        points.push(poly[seg].lerp(poly[seg + 1], t));
    }
    points
}

impl ElementCx<'_, '_> {
    /// Draw all borders for a node
    pub(crate) fn draw_border(&self, scene: &mut impl PaintScene) {
        // raea: no side has a width (Tailwind's preflight gives every element
        // `border: 0 solid`): nothing to draw, and no path built for it each frame
        let bw = self.frame.border_width;
        if bw.x0 <= 0.0 && bw.y0 <= 0.0 && bw.x1 <= 0.0 && bw.y1 <= 0.0 {
            return;
        }
        let style = &*self.style;
        let border = style.get_border();
        let current_color = style.clone_color();

        // Fast path: a uniform-width, single-color solid border whose corners are
        // all circular arcs (equal x and y radius — sharp rectangles, ordinary
        // rounded rectangles and circles all qualify) can be drawn as a single
        // stroke along its center-line instead of a fill per edge. Besides being
        // faster, this works around Vello seam artifacts at the quarter-arc joins
        // when a thin circular/elliptical border is drawn as the
        // border-box/padding-box annulus.
        //
        // A full ellipse (`border-radius: 50%` on a non-square box) has the same
        // seam problem but no rounded-rect representation, so it is stroked as an
        // ellipse.
        let all_solid = border.border_top_style == BorderStyle::Solid
            && border.border_right_style == BorderStyle::Solid
            && border.border_bottom_style == BorderStyle::Solid
            && border.border_left_style == BorderStyle::Solid;
        // raea: inside a layer warped by a perspective (B3 H5) a border is filled per edge, not stroked: a stroked
        // border came out several texels too thick along one edge of the warped layer (WPT
        // perspective-transforms-equivalence's black wedge; why the stroke does is open, B3 H5).
        let warped = crate::render::IN_WARP.with(|w| w.get() > 0);
        if all_solid && !warped {
            let color = border
                .border_top_color
                .resolve_to_absolute(&current_color)
                .as_srgb_color();
            let same_color = [
                &border.border_right_color,
                &border.border_bottom_color,
                &border.border_left_color,
            ]
            .iter()
            .all(|c| c.resolve_to_absolute(&current_color).as_srgb_color() == color);
            let bw = self.frame.border_width;
            let uniform_width = bw.x0 == bw.y0 && bw.x0 == bw.x1 && bw.x0 == bw.y1 && bw.x0 > 0.0;
            if same_color && uniform_width && color.components[3] > 0.0 {
                let bb = self.frame.border_box;
                let pb = self.frame.padding_box;
                let width = (bb.width() - pb.width()) / 2.0;
                // Miter join keeps sharp corners square. It has no effect on rounded
                // corners, whose arcs meet the straight edges tangentially.
                let stroke = Stroke::new(width).with_join(Join::Miter);

                if self.frame.is_uniform_corner_border() {
                    let radii = self.frame.border_radii;
                    // The stroke matches the CSS border shape only when each corner is
                    // sharp or its radius is at least the border width (outer radius
                    // `r`, inner radius `r - width`, concentric). For 0 < r < width CSS
                    // draws a rounded outer edge against a sharp inner corner, which a
                    // stroke can't represent — leave those to the per-edge fill.
                    let is_exact = |r: Vec2| r.x == 0.0 || r.x >= width;
                    let stroke_is_exact = is_exact(radii.top_left)
                        && is_exact(radii.top_right)
                        && is_exact(radii.bottom_right)
                        && is_exact(radii.bottom_left);
                    if stroke_is_exact {
                        let half_width = width / 2.0;
                        // Center-line rect, and corner radii shrunk to match.
                        let centerline_box = bb - Insets::uniform(half_width);
                        let centerline_radius = |r: Vec2| (r.x - half_width).max(0.0);
                        let radii = kurbo::RoundedRectRadii::new(
                            centerline_radius(radii.top_left),
                            centerline_radius(radii.top_right),
                            centerline_radius(radii.bottom_right),
                            centerline_radius(radii.bottom_left),
                        );
                        let rr = kurbo::RoundedRect::from_rect(centerline_box, radii);
                        scene.stroke(&stroke, self.transform, color, None, &rr);
                        return;
                    }
                } else if self.frame.is_elliptical_border() {
                    // Center-line radii, straight from the boxes.
                    let rx = (bb.width() + pb.width()) / 4.0;
                    let ry = (bb.height() + pb.height()) / 4.0;
                    let e = kurbo::Ellipse::new(bb.center(), (rx, ry), 0.0);
                    scene.stroke(&stroke, self.transform, color, None, &e);
                    return;
                }
            }
        }

        // (colour, path) pairs to be filled. Several entries may share a colour;
        // they are grouped before filling so that adjacent same-coloured regions
        // are drawn together, avoiding anti-aliasing seams between them.
        //
        // At most 8 entries: 4 edges, each of which can contribute 2 (the outer
        // and inner halves of a `groove`/`ridge`).
        let mut borders: SmallVec<[(Color, BezPath); 8]> = SmallVec::new();

        for &edge in &[Edge::Top, Edge::Right, Edge::Bottom, Edge::Left] {
            let (color, edge_style) = match edge {
                Edge::Top => (&border.border_top_color, border.border_top_style),
                Edge::Right => (&border.border_right_color, border.border_right_style),
                Edge::Bottom => (&border.border_bottom_color, border.border_bottom_style),
                Edge::Left => (&border.border_left_color, border.border_left_style),
            };
            let color = color.resolve_to_absolute(&current_color).as_srgb_color();

            // raea: a side without width has no area to fill
            if color.components[3] <= 0.0 || self.edge_width(edge) <= 0.0 {
                continue;
            }

            match edge_style {
                // `none`/`hidden` produce a zero-width border during layout, but
                // guard against drawing them anyway.
                BorderStyle::None | BorderStyle::Hidden => {}

                // Dashed and dotted edges are drawn immediately as their own
                // (clipped) shapes rather than being batched with the solid edges.
                BorderStyle::Dotted => self.draw_dotted_border_edge(scene, edge, color),
                BorderStyle::Dashed => self.draw_dashed_border_edge(scene, edge, color),

                // A double border is two solid lines separated by a gap, splitting
                // the border width into three equal parts (outer line / gap / inner
                // line). Both lines share a color, so both rings are placed into a
                // single path.
                BorderStyle::Double => {
                    // Needs at least 3px (one device pixel per line and per gap) to
                    // render as two lines; thinner borders fall back to a solid
                    // fill, matching browser behaviour.
                    let path = if self.edge_width(edge) < 3.0 * self.scale {
                        self.frame.border_edge_shape(edge)
                    } else {
                        let mut path = self
                            .frame
                            .border_slice(0.0, 1.0 / 3.0)
                            .border_edge_shape(edge);
                        path.extend(
                            &self
                                .frame
                                .border_slice(2.0 / 3.0, 1.0)
                                .border_edge_shape(edge),
                        );
                        path
                    };
                    borders.push((color, path));
                }

                // `inset`/`outset` are solid, but each edge is shaded lighter or
                // darker to give a bevelled (3D) appearance.
                BorderStyle::Inset | BorderStyle::Outset => {
                    let inset = edge_style == BorderStyle::Inset;
                    let shade = beveled_edge_color(color, edge, inset);
                    borders.push((shade, self.frame.border_edge_shape(edge)));
                }

                // `groove`/`ridge` split each edge into an outer and an inner half,
                // each shaded as if it were `inset`/`outset`, producing a carved or
                // raised ridge.
                BorderStyle::Groove | BorderStyle::Ridge => {
                    let ridge = edge_style == BorderStyle::Ridge;
                    let (outer, inner) = grooved_edge_colors(color, edge, ridge);
                    borders.push((
                        outer,
                        self.frame.border_slice(0.0, 0.5).border_edge_shape(edge),
                    ));
                    borders.push((
                        inner,
                        self.frame.border_slice(0.5, 1.0).border_edge_shape(edge),
                    ));
                }

                // Solid fills the whole edge region with the border colour.
                BorderStyle::Solid => {
                    borders.push((color, self.frame.border_edge_shape(edge)));
                }
            }
        }

        if borders.is_empty() {
            return;
        }

        // Group together identical colors by sorting, then fill each group as a
        // single path.
        borders.sort_unstable_by(|a, b| {
            a.0.components
                .partial_cmp(&b.0.components)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut start = 0;
        while start < borders.len() {
            let color = borders[start].0;
            // raea: the same-coloured edges' run, merged into a path sized for all of them (one allocation)
            let end = start + borders[start..].iter().take_while(|b| b.0 == color).count();
            let len = borders[start..end].iter().map(|b| b.1.elements().len()).sum();
            let mut path = BezPath::with_capacity(len);
            for b in &borders[start..end] {
                path.extend(&b.1);
            }
            let next = end;
            scene.fill(Fill::NonZero, self.transform, color, None, &path);
            start = next;
        }
    }

    /// The border width (in device pixels) of a single edge.
    fn edge_width(&self, edge: Edge) -> f64 {
        match edge {
            Edge::Top => self.frame.border_width.y0,
            Edge::Bottom => self.frame.border_width.y1,
            Edge::Left => self.frame.border_width.x0,
            Edge::Right => self.frame.border_width.x1,
        }
    }

    /// raea: the line Blink strokes for a dashed or dotted side with square
    /// corners: through the middle of the side, corner to corner of the border box.
    /// Returns its start, end and length.
    fn edge_line(&self, edge: Edge) -> Option<(Point, Point, f64)> {
        let bb = self.frame.border_box;
        let half = self.edge_width(edge) / 2.0;
        let (start, end, length) = match edge {
            Edge::Top => (
                Point::new(bb.x0, bb.y0 + half),
                Point::new(bb.x1, bb.y0 + half),
                bb.width(),
            ),
            Edge::Bottom => (
                Point::new(bb.x0, bb.y1 - half),
                Point::new(bb.x1, bb.y1 - half),
                bb.width(),
            ),
            Edge::Left => (
                Point::new(bb.x0 + half, bb.y0),
                Point::new(bb.x0 + half, bb.y1),
                bb.height(),
            ),
            Edge::Right => (
                Point::new(bb.x1 - half, bb.y0),
                Point::new(bb.x1 - half, bb.y1),
                bb.height(),
            ),
        };
        (length > 0.0).then_some((start, end, length))
    }

    /// raea: the rounded centre line Blink strokes for a dashed or dotted border
    /// (DrawCurvedDashedDottedBoxSide), the length it fits the pattern to (Blink
    /// truncates it to an integer), and how far along our path Blink's starts: on
    /// the top edge just after the top-left corner (Skia's addRRect, start 0),
    /// where ours starts at that corner.
    fn rounded_centerline(&self) -> Option<(BezPath, f64, f64)> {
        let mut centerline = self.frame.border_slice(0.0, 0.5).padding_box_bezpath();
        centerline.close_path();
        let length = centerline.perimeter(0.1).trunc();
        if length <= 0.0 {
            return None;
        }
        let start = centerline
            .segments()
            .take_while(|seg| !matches!(seg, PathSeg::Line(_)))
            .map(|seg| seg.arclen(0.1))
            .sum();
        Some((centerline, length, start))
    }

    /// Draw a single `dashed` border edge, as Blink does.
    ///
    /// The dashes are produced by stroking the centre line of the border with a
    /// dash pattern (butt caps give them flat, square ends). A square-cornered
    /// side is one line, corner to corner, whose gap is fitted so it starts and
    /// ends on a dash; a rounded border is one closed path round the whole box,
    /// its gap fitted to the perimeter, starting after the top-left corner.
    /// Dashes keep their length (3× the thickness below 3px, else 2×); lines too
    /// short for that are drawn solid or as two shortened dashes. Everything is
    /// clipped to the edge's region (the same trapezoid used by the solid path)
    /// so corners are mitred, adjacent edges of different colors don't overlap,
    /// and any border-radius is respected.
    fn draw_dashed_border_edge(&self, scene: &mut impl PaintScene, edge: Edge, color: Color) {
        let thickness = self.edge_width(edge);
        if thickness <= 0.0 {
            return;
        }

        let (dash_ratio, gap_ratio) = dashed_ratios(thickness, self.scale);
        let (dash, gap) = (dash_ratio * thickness, gap_ratio * thickness);
        let (path, dashes, offset) = if self.frame.has_border_radius() {
            let Some((centerline, length, start)) = self.rounded_centerline() else {
                return;
            };
            let dashes = blink_dashes(length, dash, gap, true, true);
            let offset = dashes.map_or(0.0, |(d, g)| dash_phase(start, d + g));
            (centerline, dashes, offset)
        } else {
            let Some((start, end, length)) = self.edge_line(edge) else {
                return;
            };
            let mut line = BezPath::new();
            line.move_to(start);
            line.line_to(end);
            (line, blink_dashes(length.round(), dash, gap, true, false), 0.0)
        };

        let mut stroke = Stroke::new(thickness).with_caps(Cap::Butt);
        if let Some((dash, gap)) = dashes {
            stroke = stroke.with_dashes(offset, [dash, gap]);
        }
        let clip = self.frame.border_edge_shape(edge);
        scene.push_clip_layer(Fill::NonZero, self.transform, &clip);
        scene.stroke(&stroke, self.transform, color, None, &path);
        scene.pop_layer();
    }

    /// Draw a single `dotted` border edge, as Blink does.
    ///
    /// Up to 3px thick the dots are squares: dashes as long as the line is thick,
    /// with equal gaps. A square-cornered side gets a whole dot in each corner,
    /// with a pixel taken from or added to the gaps next to them depending on the
    /// side's length (Blink's EnforceDotsAtEndpoints); a rounded border is one
    /// closed dash pattern starting after the top-left corner. Thicker dots are
    /// circles (diameter == thickness), their spacing fitted to the side, or to
    /// the perimeter of a rounded border. Circles are drawn as shapes because
    /// kurbo doesn't emit the zero-length dashes Blink strokes them with.
    /// Everything is clipped to the edge's region, as for the other styles.
    fn draw_dotted_border_edge(&self, scene: &mut impl PaintScene, edge: Edge, color: Color) {
        let thickness = self.edge_width(edge);
        if thickness <= 0.0 {
            return;
        }
        let radius = thickness / 2.0;
        let square = thickness <= 3.0 * self.scale;
        // Blink's dots are 0-length dashes with round caps, a hundredth of a
        // pixel closer than their fitted spacing so the last one isn't dropped.
        const EPSILON: f64 = 1.0e-2;

        let clip = self.frame.border_edge_shape(edge);
        scene.push_clip_layer(Fill::NonZero, self.transform, &clip);

        if self.frame.has_border_radius() {
            if let Some((centerline, length, start)) = self.rounded_centerline() {
                if square {
                    let mut stroke = Stroke::new(thickness).with_caps(Cap::Butt);
                    if let Some((dash, gap)) =
                        blink_dashes(length, thickness, thickness, false, true)
                    {
                        stroke = stroke.with_dashes(dash_phase(start, dash + gap), [dash, gap]);
                    }
                    scene.stroke(&stroke, self.transform, color, None, &centerline);
                } else {
                    let (spacing, count) = if length < 2.0 * thickness {
                        (length, 1)
                    } else {
                        let spacing =
                            best_dash_gap(length, thickness, thickness, true) + thickness - EPSILON;
                        (spacing, (length / spacing).floor() as usize + 1)
                    };
                    let mut path = BezPath::new();
                    for center in sample_points_along_path(&centerline, start, spacing, count) {
                        path.extend(Circle::new(center, radius).path_elements(0.1));
                    }
                    scene.fill(Fill::NonZero, self.transform, color, None, &path);
                }
            }
        } else if let Some((p1, p2, length)) = self.edge_line(edge) {
            let dir = (p2 - p1) / length;
            let across = Vec2::new(-dir.y, dir.x) * radius;
            let at = |s: f64| p1 + dir * s;
            let mut path = BezPath::new();
            if square {
                let width = thickness.round().max(1.0) as i64;
                let len = length.round() as i64;
                let w = width as f64;
                // Blink's EnforceDotsAtEndpoints: whole dots in both corners.
                let (mod4, mod6) = (len % 4, len % 6);
                let mut start_dot = None;
                let mut end_dot = None;
                if (width == 1 && len % 2 == 0) || (width == 3 && mod6 == 0) {
                    start_dot = Some((1.0, 1.0)); // (growth, offset of the first gap)
                }
                if (width == 2 && (mod4 == 0 || mod4 == 1)) || (width == 3 && (mod6 == 1 || mod6 == 2)) {
                    start_dot = Some((0.0, -1.0));
                }
                if (width == 2 && mod4 == 0) || (width == 3 && mod6 == 1) {
                    end_dot = Some(0.0); // growth
                }
                if (width == 2 && mod4 == 3) || (width == 3 && (mod6 == 4 || mod6 == 5)) {
                    start_dot = Some((0.0, 1.0));
                }
                if width == 3 && mod6 == 5 {
                    end_dot = Some(0.0);
                } else if width == 3 && mod6 == 0 {
                    end_dot = Some(1.0);
                }

                let (mut a, mut b) = (0.0, length);
                let rect = |s0: f64, s1: f64| Rect::from_points(at(s0) - across, at(s1) + across);
                if let Some((growth, offset)) = start_dot {
                    path.extend(rect(0.0, w + growth).path_elements(0.1));
                    a = 2.0 * w + offset;
                }
                if let Some(growth) = end_dot {
                    path.extend(rect(length - w - growth, length).path_elements(0.1));
                    b = length - (w + growth + 1.0);
                }
                scene.fill(Fill::NonZero, self.transform, color, None, &path);

                if b > a {
                    let mut line = BezPath::new();
                    line.move_to(at(a));
                    line.line_to(at(b));
                    let mut stroke = Stroke::new(thickness).with_caps(Cap::Butt);
                    if let Some((dash, gap)) =
                        blink_dashes(len as f64, thickness, thickness, false, false)
                    {
                        stroke = stroke.with_dashes(0.0, [dash, gap]);
                    }
                    scene.stroke(&stroke, self.transform, color, None, &line);
                }
            } else {
                // Round dots, the first and last a radius in from the corners.
                let (a, b) = (radius, length - radius);
                let spacing = if length < 2.0 * thickness {
                    f64::INFINITY
                } else {
                    best_dash_gap(length, thickness, thickness, false) + thickness - EPSILON
                };
                let mut s = a;
                while s <= b + 1e-6 {
                    path.extend(Circle::new(at(s), radius).path_elements(0.1));
                    s += spacing;
                }
                scene.fill(Fill::NonZero, self.transform, color, None, &path);
            }
        }

        scene.pop_layer();
    }

    pub(crate) fn draw_table_borders(&self, scene: &mut impl PaintScene) {
        let SpecialElementData::TableRoot(table) = &self.element.special_data else {
            return;
        };
        // Borders are only handled at the table level when BorderCollapse::Collapse
        if table.border_collapse != BorderCollapse::Collapse {
            return;
        }

        let Some(grid_info) = &mut *table.computed_grid_info.borrow_mut() else {
            return;
        };
        let Some(border_style) = table.border_style.as_deref() else {
            return;
        };

        let outer_border_style = self.style.get_border();

        let cols = PhysicalTracks::from_tracks(&grid_info.columns);
        let rows = PhysicalTracks::from_tracks(&grid_info.rows);

        // TODO: support different colors for different borders
        let current_color = self.style.clone_color();
        let border_color = border_style
            .border_top_color
            .resolve_to_absolute(&current_color)
            .as_srgb_color();

        // No need to draw transparent borders (as they won't be visible anyway)
        if border_color == Color::TRANSPARENT {
            return;
        }

        // Border widths are not adjusted for border-style in computed styles,
        // so a border with `none`/`hidden` style must be treated as zero-width.
        if border_style.border_top_style.none_or_hidden() {
            return;
        }

        // The borders are laid out as the gutters between the tracks, and as the border
        // of the table around them. Track positions are relative to the table's border box.
        let border_width = table.style.gap.map(|gap| gap.into_raw().value() as f64);
        let left = cols.origin() as f64 - border_width.width;
        let right = (cols.origin() + cols.span()) as f64 + border_width.width;
        let top = rows.origin() as f64 - border_width.height;
        let bottom = (rows.origin() + rows.span()) as f64 + border_width.height;

        let mut fill = |x0: f64, y0: f64, x1: f64, y1: f64| {
            let shape = Rect::new(x0, y0, x1, y1).scale_from_origin(self.scale);
            scene.fill(Fill::NonZero, self.transform, border_color, None, &shape);
        };

        // Horizontal borders: the outer edges, then the gutters between adjacent row tracks.
        // The vertical borders are drawn between them so that no corner is painted twice.
        if outer_border_style.border_top_style != BorderStyle::Hidden {
            fill(left, top, right, top + border_width.height);
        }
        if outer_border_style.border_bottom_style != BorderStyle::Hidden {
            fill(left, bottom - border_width.height, right, bottom);
        }
        for (prev, next) in rows.iter().zip(rows.iter().skip(1)) {
            fill(left, prev.end as f64, right, next.start as f64);
        }

        // Vertical borders, one segment per row
        for row in rows.iter() {
            let (y0, y1) = (row.start as f64, row.end as f64);
            if outer_border_style.border_left_style != BorderStyle::Hidden {
                fill(left, y0, left + border_width.width, y1);
            }
            if outer_border_style.border_right_style != BorderStyle::Hidden {
                fill(right - border_width.width, y0, right, y1);
            }
            for (prev, next) in cols.iter().zip(cols.iter().skip(1)) {
                fill(prev.end as f64, y0, next.start as f64, y1);
            }
        }
    }

    /// raea: a collapsed table's row borders (`tr { border-bottom }`, shadcn's and Basecoat's row dividers, and a cell's
    /// own, `td { border-bottom }`, a prose table's), as Chrome draws them: between two rows the widest of the upper
    /// row's bottoms and the lower row's tops (the upper's on a tie, in its color), centred on the rows' boundary, half
    /// in each (`build_table_context` gives each cell its half); the first row's top and last row's bottom centred on
    /// the table's edge, half in its cells. Across the table's columns.
    pub(crate) fn draw_table_row_borders(&self, scene: &mut impl PaintScene) {
        let SpecialElementData::TableRoot(table) = &self.element.special_data else {
            return;
        };
        if table.border_collapse != BorderCollapse::Collapse {
            return;
        }
        let Some(grid_info) = &*table.computed_grid_info.borrow() else {
            return;
        };
        let rows = PhysicalTracks::from_tracks(&grid_info.rows);
        if rows.iter().len() != table.rows.len() {
            return;
        }
        let cols = PhysicalTracks::from_tracks(&grid_info.columns);
        let gap = table.style.gap.map(|gap| gap.into_raw().value() as f64);
        let left = cols.origin() as f64 - gap.width;
        let right = (cols.origin() + cols.span()) as f64 + gap.width;

        // (width, color) of each row's top and bottom border
        let side = |width: f32, style: BorderStyle, color: &style::values::computed::Color, current: &style::color::AbsoluteColor| {
            let width = if style.none_or_hidden() { 0.0 } else { width as f64 };
            (width, color.resolve_to_absolute(current).as_srgb_color())
        };
        let none = (0.0, Color::TRANSPARENT);
        let sides = |id| {
            self.context.dom.get_node(id).and_then(|n| n.primary_styles()).map(|s| {
                let b = s.get_border();
                let current = s.clone_color();
                (
                    side(b.border_top_width.0.to_f32_px(), b.border_top_style, &b.border_top_color, &current),
                    side(b.border_bottom_width.0.to_f32_px(), b.border_bottom_style, &b.border_bottom_color, &current),
                )
            })
        };
        let mut edges: Vec<_> = table.rows.iter().map(|row| sides(row.node_id).unwrap_or((none, none))).collect();
        // a cell's own top and bottom, where wider than its row's (the row's on a tie)
        for cell in &table.cells {
            let (Some(e), Some((top, bottom))) = (edges.get_mut(cell.row as usize), sides(cell.node_id)) else { continue };
            if top.0 > e.0.0 {
                e.0 = top;
            }
            if bottom.0 > e.1.0 {
                e.1 = bottom;
            }
        }
        let tracks: Vec<_> = rows.iter().collect();
        let mut fill = |y0: f64, y1: f64, color: Color| {
            if y1 > y0 && color != Color::TRANSPARENT {
                let shape = Rect::new(left, y0, right, y1).scale_from_origin(self.scale);
                scene.fill(Fill::NonZero, self.transform, color, None, &shape);
            }
        };
        for (i, track) in tracks.iter().enumerate() {
            let (top, bottom) = edges[i];
            if i == 0 {
                fill(track.start as f64 - top.0 / 2.0, track.start as f64 + top.0 / 2.0, top.1);
            }
            match tracks.get(i + 1) {
                None => fill(track.end as f64 - bottom.0 / 2.0, track.end as f64 + bottom.0 / 2.0, bottom.1),
                Some(next) => {
                    let below = edges[i + 1].0;
                    let (width, color) = if below.0 > bottom.0 { below } else { bottom };
                    fill(track.end as f64 - width / 2.0, next.start as f64 + width / 2.0, color);
                }
            }
        }
    }

    /// ❌ dotted - Defines a dotted border
    /// ❌ dashed - Defines a dashed border
    /// ✅ solid - Defines a solid border
    /// ❌ double - Defines a double border
    /// ❌ groove - Defines a 3D grooved border. The effect depends on the border-color value
    /// ❌ ridge - Defines a 3D ridged border. The effect depends on the border-color value
    /// ❌ inset - Defines a 3D inset border. The effect depends on the border-color value
    /// ❌ outset - Defines a 3D outset border. The effect depends on the border-color value
    /// ✅ none - Defines no border
    /// ✅ hidden - Defines a hidden border
    /// raea: `outline-style: auto` as Chrome 152 on macOS draws it (Blitz drew nothing): a 2px ring, its outer edge on
    /// a text field's or select's border box and 1px out from anything else's, its outer corners the box's radii plus
    /// that 1px and 4px at least.
    fn draw_focus_ring(&self, scene: &mut impl PaintScene, color: peniko::Color) {
        let field = self.text_input.is_some() || &*self.element.name.local == "select";
        let d = if field { 0.0 } else { self.scale };
        let w = 2.0 * self.scale;
        let outer = self.frame.border_box.inflate(d, d);
        let r = self.frame.border_radii;
        let min = 4.0 * self.scale;
        let ro = kurbo::RoundedRectRadii::new((r.top_left.x + d).max(min), (r.top_right.x + d).max(min), (r.bottom_right.x + d).max(min), (r.bottom_left.x + d).max(min));
        let ri = kurbo::RoundedRectRadii::new((ro.top_left - w).max(0.0), (ro.top_right - w).max(0.0), (ro.bottom_right - w).max(0.0), (ro.bottom_left - w).max(0.0));
        let mut path = outer.to_rounded_rect(ro).to_path(0.1);
        path.extend(outer.inflate(-w, -w).to_rounded_rect(ri).to_path(0.1));
        scene.fill(Fill::EvenOdd, self.transform, color, None, &path);
    }

    pub(crate) fn draw_outline(&self, scene: &mut impl PaintScene) {
        let outline = self.style.get_outline();

        let current_color = self.style.clone_color();
        let color = outline
            .outline_color
            .resolve_to_absolute(&current_color)
            .as_srgb_color();

        let style = match outline.outline_style {
            OutlineStyle::Auto => return self.draw_focus_ring(scene, color),
            OutlineStyle::BorderStyle(style) => style,
        };

        let path = match style {
            BorderStyle::None | BorderStyle::Hidden => return,
            BorderStyle::Solid => self.frame.outline(),

            // TODO: Implement other border styles
            BorderStyle::Inset
            | BorderStyle::Groove
            | BorderStyle::Outset
            | BorderStyle::Ridge
            | BorderStyle::Dotted
            | BorderStyle::Dashed
            | BorderStyle::Double => self.frame.outline(),
        };

        // raea: even-odd, the ring between its two rounded rects (CssBox::outline)
        scene.fill(Fill::EvenOdd, self.transform, color, None, &path);
    }
}
