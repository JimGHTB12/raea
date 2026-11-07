//! Text widths as a browser has them (R5a). Rune's own measure rounds a text's width up to whole device pixels
//! (rune src/text.rs `measure`, to keep layout on the pixel grid), so every box that shrinks to fit a text comes out up
//! to a pixel wider than Chrome's, which keeps the fraction. The host measures each text with a TextSystem of its own
//! over the same embedded fonts and caps the text at that width; Rune's layout then has the unrounded width.

use std::cell::RefCell;

use rune::text::TextSystem;
use rune::TextStyle;

#[derive(Default)]
pub struct Measure(RefCell<Option<TextSystem>>);

impl Measure {
    /// `s`'s width on one line, unrounded, in points.
    pub fn width(&self, s: &str, style: TextStyle) -> f32 {
        let mut ts = self.0.borrow_mut();
        let ts = ts.get_or_insert_with(|| TextSystem::new(&[], ("Inter", "Inter")));
        ts.layout_rich(s, &[], style, 1.0, None).full_width()
    }
}
