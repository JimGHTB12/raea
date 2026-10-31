//! A Blitz custom widget, written as for Blitz alone (blitz-dom's Widget, anyrender's Scene; no Bevy, no raea): an
//! `<x-swatch color="#rrggbb">` fills its box with its colour (magenta until it has one). The plugin catalog
//! (sdk/raea/bench/bevy-plugins/package.json) declares it in raea.plugins
//! (`"widgets": { "x-swatch": "raea_bench_swatch::Swatch::default()" }`), so raea's pages draw it (docs/sdk.feature S8b).

use anyrender::{PaintRef, PaintScene, RenderContext, Scene};
use blitz_dom::Widget;
use blitz_dom::node::ComputedStyles;
use kurbo::{Affine, Rect};
use peniko::{Color, Fill};

/// The swatch: its colour, from its `color` attribute
pub struct Swatch {
    color: Color,
}

impl Default for Swatch {
    fn default() -> Self {
        Swatch { color: Color::from_rgb8(255, 0, 255) }
    }
}

/// `#rrggbb` as a colour
fn hex(s: &str) -> Option<Color> {
    let s = s.strip_prefix('#')?;
    let n = u32::from_str_radix(s, 16).ok().filter(|_| s.len() == 6)?;
    Some(Color::from_rgb8((n >> 16) as u8, (n >> 8) as u8, n as u8))
}

impl Widget for Swatch {
    fn attribute_changed(&mut self, name: &str, _old: Option<&str>, new: Option<&str>) {
        if let Some(color) = new.and_then(hex).filter(|_| name == "color") {
            self.color = color;
        }
    }

    fn paint(&mut self, _: &mut dyn RenderContext, _: &ComputedStyles, width: u32, height: u32, _: f64) -> Scene {
        let mut scene = Scene::new();
        scene.fill(Fill::NonZero, Affine::IDENTITY, PaintRef::Solid(self.color), None, &Rect::new(0.0, 0.0, width as f64, height as f64));
        scene
    }
}
