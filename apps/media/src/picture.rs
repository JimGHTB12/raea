//! `media still` and `media sheet`: frames as images.

use super::*;

pub(crate) fn to_image(f: Frame) -> R<image::RgbaImage> {
    image::RgbaImage::from_raw(f.width, f.height, f.rgba).ok_or_else(|| "a decoded frame had the wrong size".to_string())
}

pub(crate) fn save(img: &image::RgbaImage, out: &Path) -> R<()> {
    if let Some(d) = out.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    let r = match ext(out).as_str() {
        "jpg" | "jpeg" => image::DynamicImage::ImageRgba8(img.clone()).to_rgb8().save(out),
        _ => img.save(out),
    };
    r.map_err(|e| format!("{}: {e}", out.display()))
}

/// The frame showing at `t` (the last frame when `t` is past the end).
pub(crate) fn frame(path: &Path, t: f64) -> R<image::RgbaImage> {
    let mut d = backend()?.open_video(path)?;
    let dur = d.info().duration;
    let t = t.clamp(0.0, (dur - 0.5 / d.info().fps.max(1.0)).max(0.0));
    to_image(d.frame_at(t)?)
}

pub fn still(path: &Path, at: f64, out: &Path) -> R<Value> {
    let img = if IMAGE_EXTS.contains(&ext(path).as_str()) {
        image::open(path).map_err(|e| format!("{}: {e}", path.display()))?.to_rgba8()
    } else {
        frame(path, at)?
    };
    save(&img, out)?;
    Ok(json!({"wrote": out, "at": at, "width": img.width(), "height": img.height()}))
}

pub fn sheet(files: &[PathBuf], n: usize, w: u32, out: &Path) -> R<Value> {
    if files.is_empty() {
        return Err("raea media sheet FILE... -o OUT: which files?".into());
    }
    let mut tiles = Vec::new();
    if files.len() == 1 {
        let p = probe_of(&files[0])?;
        for i in 0..n.max(1) {
            let t = (i as f64 + 0.5) / n.max(1) as f64 * p.duration;
            tiles.push((t, frame(&files[0], t)?));
        }
    } else {
        for f in files {
            let p = probe_of(f)?;
            let img = if p.kind == "image" { image::open(f).map_err(|e| format!("{}: {e}", f.display()))?.to_rgba8() } else { frame(f, p.duration / 2.0)? };
            tiles.push((p.duration / 2.0, img));
        }
    }
    let th = tiles.iter().map(|(_, i)| (i.height() as f64 * w as f64 / i.width() as f64).round() as u32).max().unwrap_or(w);
    let cols = tiles.len().min(4) as u32;
    let rows = (tiles.len() as u32).div_ceil(cols);
    let mut sheet = image::RgbaImage::from_pixel(cols * w, rows * th, image::Rgba([0, 0, 0, 255]));
    for (k, (_, img)) in tiles.iter().enumerate() {
        let h = (img.height() as f64 * w as f64 / img.width() as f64).round() as u32;
        let small = image::imageops::resize(img, w, h.max(1), image::imageops::FilterType::Triangle);
        image::imageops::overlay(&mut sheet, &small, ((k as u32 % cols) * w) as i64, ((k as u32 / cols) * th) as i64);
    }
    save(&sheet, out)?;
    Ok(json!({"wrote": out, "tiles": tiles.len(), "at": tiles.iter().map(|(t, _)| round3(*t)).collect::<Vec<_>>(), "width": sheet.width(), "height": sheet.height()}))
}

// ---- sound
