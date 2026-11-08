//! bevy_text's glyph atlases for a text (Bevy's ui/text/font_atlas_debug shows them): the text shaped in one face at
//! one size as bevy_text shapes it, and each glyph its layout meets, in order, rasterized and packed by bevy_text's own
//! `add_glyph_to_atlas` (swash's alpha mask, 2 px padding, 512 px pages and up), as Bevy's FontAtlasSet holds them for
//! that face and size. Hinting off and smoothing on, Bevy's defaults.

use bevy::asset::Assets;
use bevy::image::Image;
use bevy::text::{FontSmoothing, GlyphCacheKey, add_glyph_to_atlas, get_glyph_atlas_info};
use parley::fontique::Blob;
use parley::{FontContext, LayoutContext, PositionedLayoutItem, StyleProperty};

/// Each atlas page bevy_text fills for `text` in the face `font` (a font file's bytes) at `size` px: its width, height
/// and RGBA (Rgba8UnormSrgb: white, the glyphs' coverage in alpha), in the order the pages were made. Glyphs from a
/// face other than `font` (a fallback's) are in that face's own atlases in Bevy, and left out here.
pub fn atlases(font: Vec<u8>, size: f32, text: &str) -> Result<Vec<(u32, u32, Vec<u8>)>, String> {
    let mut cx = FontContext::default();
    let blob = Blob::from(font);
    let blob_id = blob.id();
    let families = cx.collection.register_fonts(blob, None);
    let family = families.first().ok_or("the font file has no face")?.0;
    let name = cx.collection.family_name(family).ok_or("the font's family has no name")?.to_string();
    let mut lcx = LayoutContext::<()>::new();
    let mut b = lcx.ranged_builder(&mut cx, text, 1.0, true);
    b.push_default(StyleProperty::FontFamily(parley::FontFamily::Single(parley::FontFamilyName::Named(name.into()))));
    b.push_default(StyleProperty::FontSize(size));
    let mut layout = b.build(text);
    layout.break_all_lines(None);

    let mut textures = Assets::<Image>::default();
    let mut pages = Vec::new();
    let mut scale = swash::scale::ScaleContext::new();
    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else { continue };
            let run = glyph_run.run();
            let face = run.font();
            if face.data.id() != blob_id {
                continue;
            }
            let font_ref = swash::FontRef::from_index(face.data.as_ref(), face.index as usize).ok_or("swash can't read the face")?;
            let mut scaler = scale
                .builder_with_id(font_ref, [face.data.id(), face.index.into()])
                .size(run.font_size())
                .hint(false)
                .normalized_coords(run.normalized_coords().iter().map(|c| c.to_bits()))
                .build();
            for glyph in glyph_run.positioned_glyphs() {
                let Ok(glyph_id) = u16::try_from(glyph.id) else { continue };
                if get_glyph_atlas_info(&mut pages, GlyphCacheKey { glyph_id }).is_some() {
                    continue;
                }
                add_glyph_to_atlas(&mut pages, &mut textures, &mut scaler, FontSmoothing::AntiAliased, glyph_id).map_err(|e| format!("glyph {glyph_id}: {e}"))?;
            }
        }
    }
    pages
        .iter()
        .map(|p| {
            let image = textures.get(&p.texture).ok_or("an atlas page's texture is gone")?;
            Ok((image.width(), image.height(), image.data.clone().ok_or("an atlas page has no pixels")?))
        })
        .collect()
}
