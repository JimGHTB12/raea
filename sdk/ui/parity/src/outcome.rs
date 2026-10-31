//! A case's outcome: its diffs against Chrome and how it judges.

use super::*;

#[derive(Default)]
pub(crate) struct Outcome {
    pub(crate) layout: bool,
    pub(crate) pixels: bool,
    pub(crate) behaviour: bool,
    pub(crate) a11y: bool,
    pub(crate) error: Option<String>,
    pub(crate) steps: Vec<Value>,
    /// the first differing step's frames, kept for a failing case
    pub(crate) frames: Option<(image::RgbaImage, image::RgbaImage, String)>,
    pub(crate) ssim_min: f64,
    pub(crate) reason: Option<String>,
}

impl Outcome {
    pub(crate) fn pass(&self) -> bool {
        self.error.is_none() && self.layout && self.pixels && self.behaviour && self.a11y
    }
    pub(crate) fn check(&self, name: &str) -> bool {
        self.error.is_none()
            && match name {
                "layout" => self.layout,
                "pixels" => self.pixels,
                "behaviour" => self.behaviour,
                "a11y" => self.a11y,
                _ => false,
            }
    }
    pub(crate) fn note(&mut self, label: &str, a: &Snap, b: &Snap, d: &Diffs) {
        self.layout &= d.layout.is_empty();
        self.pixels &= d.pixels.is_empty();
        self.behaviour &= d.behaviour.is_empty();
        self.a11y &= d.a11y.is_empty();
        if let Some(s) = d.ssim {
            self.ssim_min = self.ssim_min.min(s);
        }
        let differs = !(d.layout.is_empty() && d.pixels.is_empty() && d.behaviour.is_empty() && d.a11y.is_empty());
        if differs {
            if self.frames.is_none() {
                self.frames = Some((a.png.clone(), b.png.clone(), label.to_string()));
            }
            if self.reason.is_none() {
                let first = d.layout.first().map(|s| format!("layout: {s}")).or_else(|| d.behaviour.first().map(|s| format!("behaviour: {s}"))).or_else(|| d.a11y.first().map(|s| format!("a11y: {s}"))).or_else(|| d.pixels.first().map(|s| format!("pixels: {s}")));
                self.reason = first.map(|f| format!("{label}: {f}"));
            }
            if self.steps.len() < 8 {
                let mut j = diffs_json(label, d);
                for k in ["layout", "pixels", "behaviour", "a11y"] {
                    if let Some(a) = j[k].as_array_mut() {
                        a.truncate(10);
                    }
                }
                self.steps.push(j);
            }
        }
    }
}
