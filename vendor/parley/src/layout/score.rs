// raea: `text-wrap-style: pretty | balance` as Chrome breaks them, Blink's ScoreLineBreaker
// (third_party/blink/renderer/core/layout/inline/score_line_breaker.cc): a Knuth-Plass pass over a paragraph's
// greedy lines that keeps their count. Pretty optimises a paragraph's last 4 lines, and only when the last is
// one word shorter than a third of the line; balance a paragraph of at most 6 lines, always.

use alloc::vec;
use alloc::vec::Vec;

use parley_engine::shape::Whitespace;

use crate::layout::spacing::{EffectiveSpacing, Justification};
use crate::layout::whitespace::atom_hanging_advance;
use crate::layout::{BreakReason, Layout, LayoutItemKind, Run};
use crate::style::Brush;
use crate::{InlineBoxKind, TextWrapMode};

const MAX_LINES_FOR_OPTIMAL: usize = 4;
const MAX_LINES_FOR_BALANCE: usize = 6;
const SHORT_LINE_DENOMINATOR: f32 = 3.;
const ORPHANS_PENALTY: f32 = 10000.;
const SCORE_OVERFULL: f32 = 1e12;
const SCORE_INFINITY: f32 = f32::MAX;

/// What a score pass needs of an atom (or in-flow inline box): where it starts in the text, its x along the text,
/// and whether a line may break before it.
#[derive(Clone, Copy)]
struct Unit {
    byte: usize,
    x: f32,
    advance: f32,
    hang: f32,
    soft_before: bool,
}

#[derive(Clone, Copy)]
struct Candidate {
    unit: usize,
    pos_if_break: f32,
    pos_no_break: f32,
    penalty: f32,
}

#[derive(Clone, Copy)]
struct Score {
    score: f32,
    prev: usize,
    line_index: usize,
}

impl<B: Brush> Layout<B> {
    /// raea: the line widths that make the greedy breaker break as Chrome's `text-wrap-style: pretty` (or
    /// `balance`) does, after [`Layout::break_all_lines`] (or the same breaking by hand) at `max_advance`. One width
    /// per line; `None` when the greedy lines stand. `font_size` is the block's, `zoom` the layout's scale.
    pub fn score_line_widths(
        &self,
        max_advance: f32,
        balance: bool,
        justified: bool,
        font_size: f32,
        zoom: f32,
    ) -> Option<Vec<f32>> {
        let lines = &self.data.lines;
        if lines.len() < 2 || !max_advance.is_finite() {
            return None;
        }
        let units = self.score_units()?;
        let starts: Vec<usize> = lines
            .iter()
            .map(|line| units.partition_point(|u| u.byte < line.text_range.start))
            .collect();
        let indent = self.data.indent_amount;
        let options = self.data.indent_options;
        let line_indent = |i: usize| match i.checked_sub(1).map(|p| lines[p].break_reason) {
            None => options.indent_for_first_line(indent),
            Some(BreakReason::Explicit) => options.indent_following_forced_line_break(indent),
            Some(_) => options.indent_following_soft_wrap_break(indent),
        };
        let unit_end = |i: usize| starts.get(i + 1).copied().unwrap_or(units.len());

        let mut widths = vec![max_advance; lines.len()];
        let mut changed = false;
        let mut first = 0;
        while first < lines.len() {
            let mut last = first;
            while last + 1 < lines.len() && lines[last].break_reason != BreakReason::Explicit {
                last += 1;
            }
            let count = last + 1 - first;
            let window = if balance {
                (count <= MAX_LINES_FOR_BALANCE).then_some(first)
            } else {
                Some(last + 1 - count.min(MAX_LINES_FOR_OPTIMAL))
            };
            if let Some(start) = window
                && count >= 2
                && (balance || self.last_line_short(&units, starts[last], unit_end(last), max_advance - line_indent(last)))
                && let Some(breaks) = optimize(
                    &units,
                    starts[start],
                    unit_end(last),
                    &(start..=last).map(|i| (max_advance - line_indent(i)).max(0.)).collect::<Vec<_>>(),
                    balance,
                    justified,
                    font_size,
                    zoom,
                )
            {
                for (k, (from, to)) in breaks.iter().enumerate() {
                    let i = start + k;
                    if *from != starts[i] {
                        changed = true;
                    }
                    let content = trimmed_end(&units, *from, *to) - units[*from].x;
                    widths[i] = (line_indent(i) + content + 0.01).min(max_advance);
                }
            }
            first = last + 1;
        }
        changed.then_some(widths)
    }

    /// raea: aligns every line within `max_advance` from its start, after breaking it at the narrower widths
    /// [`Layout::score_line_widths`] gave (a line's own max advance is otherwise the box it aligns in).
    pub fn set_lines_max_advance(&mut self, max_advance: f32) {
        for line in &mut self.data.lines {
            line.metrics.inline_max_coord = line.metrics.inline_min_coord + max_advance;
        }
    }

    /// Chrome's `ShouldOptimize`: the last line is shorter than a third of its width and can't break inside.
    fn last_line_short(&self, units: &[Unit], from: usize, to: usize, available: f32) -> bool {
        from < to
            && trimmed_end(units, from, to) - units[from].x < available / SHORT_LINE_DENOMINATOR
            && !units[from + 1..to].iter().any(|u| u.soft_before)
    }

    /// The text's atoms and in-flow boxes in order, with their x as one line; `None` for what this pass doesn't
    /// break (right-to-left text, floats, no wrapping).
    fn score_units(&self) -> Option<Vec<Unit>> {
        let mut units = Vec::new();
        let mut x = 0.;
        let mut soft_next = false;
        for item in &self.data.items {
            match item.kind {
                LayoutItemKind::InlineBox => {
                    let inline_box = &self.data.inline_boxes[item.index].inline_box;
                    match inline_box.kind {
                        InlineBoxKind::InFlow => {}
                        InlineBoxKind::OutOfFlow => continue,
                        InlineBoxKind::CustomOutOfFlow => return None,
                    }
                    units.push(Unit {
                        byte: inline_box.index,
                        x,
                        advance: inline_box.width,
                        hang: 0.,
                        soft_before: soft_next,
                    });
                    x += inline_box.width;
                    // raea: an inline element's edge is no soft wrap opportunity (line_break.rs)
                    if inline_box.id & crate::InlineBox::EDGE == 0 || !parley_engine::raea_edge_spaces() {
                        soft_next = true;
                    }
                }
                LayoutItemKind::TextRun => {
                    if item.bidi_level.is_rtl() {
                        return None;
                    }
                    let run = Run::new(self, 0, 0, item.index, None);
                    let slice = run.full_slice();
                    let spacing = EffectiveSpacing::new(run.data.spacing, Justification::NONE);
                    let start = self.data.shaped_text.runs()[item.index].shaped_clusters_range.start;
                    for atom in slice.atoms_from(start) {
                        let first_character = &atom.characters()[0];
                        let style = &self.data.styles[first_character.style_index as usize];
                        if style.text_wrap_mode != TextWrapMode::Wrap {
                            return None;
                        }
                        let advance = if first_character.whitespace == Whitespace::Newline {
                            0.
                        } else {
                            spacing.atom_advance(&atom)
                        };
                        let (hang, _) = atom_hanging_advance(slice, &atom, &self.data.styles, spacing, false);
                        // raea: text-autospace after the atom (autospace.rs), dropped at a line's end like a hang
                        let autospace = match self.data.autospace.is_empty() {
                            true => 0.,
                            false => self.data.autospace.get(&atom.shaped_clusters_range().start).copied().unwrap_or(0.),
                        };
                        let hang = hang.min(advance) + autospace;
                        let advance = advance + autospace;
                        units.push(Unit {
                            byte: first_character.text_byte_range().start,
                            x,
                            advance,
                            hang,
                            soft_before: soft_next || first_character.flags.is_soft_wrap_opportunity(),
                        });
                        x += advance;
                        soft_next = false;
                    }
                }
            }
        }
        Some(units)
    }
}

/// Where the units `from..to` end once their hanging whitespace is left out.
fn trimmed_end(units: &[Unit], from: usize, to: usize) -> f32 {
    let mut end = to;
    while end > from && units[end - 1].hang >= units[end - 1].advance {
        end -= 1;
    }
    match end {
        e if e == from => units[from].x,
        e => units[e - 1].x + units[e - 1].advance - units[e - 1].hang,
    }
}

/// Blink's `Optimize`: the best breaks of the units `from..to` into `widths.len()` lines, as (first unit, end unit)
/// a line; `None` when there's too little to choose from or the best takes another number of lines.
#[allow(clippy::too_many_arguments)]
fn optimize(
    units: &[Unit],
    from: usize,
    to: usize,
    widths: &[f32],
    balance: bool,
    justified: bool,
    font_size: f32,
    zoom: f32,
) -> Option<Vec<(usize, usize)>> {
    // Minikin's penalties, as Blink takes them
    let width_times_font_size = widths[widths.len() - 1] * font_size / zoom;
    let line_penalty = if justified { 0. } else { width_times_font_size * 4. };

    let mut candidates = vec![Candidate {
        unit: from,
        pos_if_break: units[from].x,
        pos_no_break: units[from].x,
        penalty: 0.,
    }];
    for i in from + 1..to {
        if units[i].soft_before {
            candidates.push(Candidate {
                unit: i,
                pos_if_break: trimmed_end(units, from, i),
                pos_no_break: units[i].x,
                penalty: 0.,
            });
        }
    }
    let end = trimmed_end(units, from, to);
    candidates.push(Candidate { unit: to, pos_if_break: end, pos_no_break: end, penalty: 0. });
    // at least 3 break opportunities besides the two ends
    if candidates.len() < 5 {
        return None;
    }
    let n = candidates.len();
    candidates[n - 2].penalty += ORPHANS_PENALTY * zoom;

    let available = |line: usize| widths[line.min(widths.len() - 1)];
    let mut scores = vec![Score { score: 0., prev: 0, line_index: 0 }];
    let mut active = 0;
    for end in 1..n {
        let end_candidate = candidates[end];
        let is_last = end == n - 1;
        let mut best = SCORE_INFINITY;
        let mut best_prev = 0;
        let mut last_line_index = scores[active].line_index;
        let mut width = available(last_line_index);
        let mut start_edge = end_candidate.pos_if_break - width;
        let mut best_hope = 0.;
        for start in active..end {
            let start_score = scores[start];
            if start_score.line_index != last_line_index {
                last_line_index = start_score.line_index;
                let new_width = available(last_line_index);
                if new_width != width {
                    width = new_width;
                    start_edge = end_candidate.pos_if_break - width;
                    best_hope = 0.;
                }
            }
            if start_score.score + best_hope >= best {
                continue;
            }
            let delta = candidates[start].pos_no_break - start_edge;
            let mut width_score = 0.;
            if (is_last || !justified) && delta < 0. {
                width_score = SCORE_OVERFULL;
            } else if is_last && !balance {
                // Blink adds 4x the start's hyphen penalty here; parley doesn't hyphenate
            } else if delta < 0. {
                width_score = SCORE_OVERFULL;
            } else {
                width_score = delta * delta / zoom;
            }
            if delta < 0. {
                active = start + 1;
            } else {
                best_hope = width_score;
            }
            let score = start_score.score + width_score;
            if score <= best {
                best = score;
                best_prev = start;
            }
        }
        scores.push(Score {
            score: best + end_candidate.penalty + line_penalty,
            prev: best_prev,
            line_index: scores[best_prev].line_index + 1,
        });
    }

    let mut breaks = Vec::new();
    let mut i = n - 1;
    while i > 0 {
        let prev = scores[i].prev;
        breaks.push((candidates[prev].unit, candidates[i].unit));
        i = prev;
    }
    breaks.reverse();
    (breaks.len() == widths.len()).then_some(breaks)
}
