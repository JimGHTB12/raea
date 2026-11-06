//! The ops raea runs itself, with no model and no server: image.crop, image.scale and image.stitch (what ComfyUI's
//! ImageCrop, ImageScale* and ImageStitch do), each writing a PNG.

use std::path::{Path, PathBuf};

use image::{DynamicImage, GenericImage, GenericImageView, imageops::FilterType};
use serde_json::Value as J;

use crate::flow::Step;

pub const OPS: [&str; 3] = ["image.crop", "image.scale", "image.stitch"];

fn open(f: &Path) -> Result<DynamicImage, String> {
    image::open(f).map_err(|e| format!("{}: {e}", f.display()))
}

/// `s` made from its pictures (`file` gives a reference's file), written to `<dir>/<key>.png`.
pub fn run(s: &Step, file: &dyn Fn(&J) -> Result<PathBuf, String>, dir: &Path, key: &str) -> Result<PathBuf, String> {
    let n = |k: &str| s.inputs.get(k).and_then(|v| v.as_f64());
    let pic = |k: &str| s.inputs.get(k).ok_or(format!("{} needs {k}", s.op)).and_then(file).and_then(|f| open(&f));
    let out = match s.op.as_str() {
        "image.crop" => {
            let img = pic("image")?;
            let (x, y) = (n("x").unwrap_or(0.0).max(0.0) as u32, n("y").unwrap_or(0.0).max(0.0) as u32);
            // ComfyUI clamps the box to the picture
            let (x, y) = (x.min(img.width().saturating_sub(1)), y.min(img.height().saturating_sub(1)));
            let w = (n("width").unwrap_or(img.width() as f64) as u32).clamp(1, img.width() - x);
            let h = (n("height").unwrap_or(img.height() as f64) as u32).clamp(1, img.height() - y);
            img.crop_imm(x, y, w, h)
        }
        "image.scale" => {
            let img = pic("image")?;
            let (iw, ih) = (img.width() as f64, img.height() as f64);
            let (w, h) = if let Some(f) = n("factor") {
                (iw * f, ih * f)
            } else if let Some(mp) = n("megapixels") {
                let f = (mp * 1_048_576.0 / (iw * ih)).sqrt();
                (iw * f, ih * f)
            } else {
                // a missing side keeps the proportions
                match (n("width"), n("height")) {
                    (Some(w), Some(h)) => (w, h),
                    (Some(w), None) => (w, ih * w / iw),
                    (None, Some(h)) => (iw * h / ih, h),
                    (None, None) => (iw, ih),
                }
            };
            let (w, h) = ((w.round() as u32).max(1), (h.round() as u32).max(1));
            match s.inputs.get("fit").and_then(|f| f.as_str()).unwrap_or("stretch") {
                "cover" => img.resize_to_fill(w, h, FilterType::Lanczos3),
                "contain" => img.resize(w, h, FilterType::Lanczos3),
                _ => img.resize_exact(w, h, FilterType::Lanczos3),
            }
        }
        "image.stitch" => {
            let list = s.inputs.get("images").and_then(|v| v.as_array()).ok_or("image.stitch needs images, a list")?;
            let pics: Vec<DynamicImage> = list.iter().map(|v| file(v).and_then(|f| open(&f))).collect::<Result<_, _>>()?;
            let dir_ = s.inputs.get("direction").and_then(|d| d.as_str()).unwrap_or("right");
            stitch(pics, dir_)?
        }
        op => return Err(format!("{op} isn't one raea runs itself ({})", OPS.join(", "))),
    };
    let f = dir.join(format!("{key}.png"));
    out.save(&f).map_err(|e| format!("{}: {e}", f.display()))?;
    Ok(f)
}

/// Pictures side by side (right, left) or stacked (down, up), each scaled to the first's height or width, as
/// ImageStitch does with match_image_size.
fn stitch(mut pics: Vec<DynamicImage>, direction: &str) -> Result<DynamicImage, String> {
    if pics.is_empty() {
        return Err("image.stitch: no images".into());
    }
    if matches!(direction, "left" | "up") {
        pics.reverse();
    }
    let across = matches!(direction, "right" | "left");
    let (w0, h0) = pics[0].dimensions();
    let pics: Vec<DynamicImage> = pics.into_iter().map(|p| {
        let (w, h) = p.dimensions();
        if across && h != h0 { p.resize_exact((w as f64 * h0 as f64 / h as f64).round() as u32, h0, FilterType::Lanczos3) }
        else if !across && w != w0 { p.resize_exact(w0, (h as f64 * w0 as f64 / w as f64).round() as u32, FilterType::Lanczos3) }
        else { p }
    }).collect();
    let (w, h) = if across { (pics.iter().map(|p| p.width()).sum(), h0) } else { (w0, pics.iter().map(|p| p.height()).sum()) };
    let mut out = DynamicImage::new_rgba8(w, h);
    let mut at = 0;
    for p in &pics {
        let (x, y) = if across { (at, 0) } else { (0, at) };
        out.copy_from(&p.to_rgba8(), x, y).map_err(|e| e.to_string())?;
        at += if across { p.width() } else { p.height() };
    }
    Ok(out)
}
