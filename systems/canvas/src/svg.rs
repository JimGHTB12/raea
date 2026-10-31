//! Whole svg documents drawn as pictures (a canvas's drawImage of an svg): usvg parses (shapes, styles, the
//! viewBox, text in the given fonts) and resvg draws, at the size asked in physical pixels, so nothing is scaled up.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use tiny_skia as sk;


/// An svg document drawn at `size`: straight-alpha sRGB RGBA, row by row (None if it doesn't parse). Its root's
/// width and height are mapped onto `size`.
pub fn raster_with(doc: &str, size: UVec2, opt: &usvg::Options) -> Option<Vec<u8>> {
    let tree = usvg::Tree::from_str(doc, opt).map_err(|e| eprintln!("canvas: svg: {e}")).ok()?;
    let mut pm = sk::Pixmap::new(size.x.max(1), size.y.max(1))?;
    let ts = tree.size();
    let t = sk::Transform::from_scale(size.x as f32 / ts.width().max(1e-6), size.y as f32 / ts.height().max(1e-6));
    resvg::render(&tree, t, &mut pm.as_mut());
    let mut out = Vec::with_capacity(pm.data().len());
    for px in pm.pixels() {
        let c = px.demultiply();
        out.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    Some(out)
}

/// A whole svg document (an img's source): its own size in CSS pixels (its width and height, else its viewBox's), or
/// None if it doesn't parse.
pub fn natural_size(doc: &[u8], opt: &usvg::Options) -> Option<Vec2> {
    let tree = usvg::Tree::from_data(doc, opt).ok()?;
    Some(Vec2::new(tree.size().width(), tree.size().height()))
}

/// A whole svg document drawn at `size` (physical pixels), as an RGBA8 sRGB image (straight alpha).
pub fn raster_doc(doc: &[u8], size: UVec2, opt: &usvg::Options) -> Option<Image> {
    let text = std::str::from_utf8(doc).ok()?;
    let px = raster_with(text, size, opt)?;
    Some(Image::new(Extent3d { width: size.x.max(1), height: size.y.max(1), depth_or_array_layers: 1 }, TextureDimension::D2, px, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::RENDER_WORLD))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The legacy app's sidebar toggle (shell/header.tsx), stroked at 1.6 in a 24-unit box: crisp at 1x and 2x, its stroke
    /// twice as wide at 2x, not blurred up
    #[test]
    fn crisp_at_scale() {
        let doc = |px: u32| {
            format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" width=\"{px}\" height=\"{px}\" viewBox=\"0 0 24 24\" color=\"#ffffff\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.6\"><rect x=\"3\" y=\"4\" width=\"18\" height=\"16\" rx=\"2\"/><path d=\"M9 4v16\"/></svg>"
            )
        };
        let at = |px: &[u8], w: u32, x: u32, y: u32| px[((y * w + x) * 4 + 3) as usize];
        let one = raster_with(&doc(24), UVec2::splat(24), &usvg::Options::default()).unwrap();
        let two = raster_with(&doc(48), UVec2::splat(48), &usvg::Options::default()).unwrap();
        // the divider, 1.6 units wide about x = 9: two pixels 80% covered at 1x; at 2x two pixels whole and their
        // neighbours 60%, then nothing a unit away, at either size
        assert!(at(&one, 24, 8, 12) > 150 && at(&one, 24, 9, 12) > 150, "1x divider {}", at(&one, 24, 9, 12));
        assert!(at(&two, 48, 17, 24) == 255 && at(&two, 48, 18, 24) == 255, "2x divider {}", at(&two, 48, 18, 24));
        assert_eq!(at(&one, 24, 12, 12), 0);
        assert_eq!(at(&two, 48, 21, 24), 0, "2x isn't 1x blurred up");
        // the white stroke reaches the image as white
        let i = ((12 * 24 + 9) * 4) as usize;
        assert_eq!(&one[i..i + 3], &[255, 255, 255]);
    }
}
