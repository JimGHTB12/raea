//! An image file decoded to an RGBA8 sRGB picture, for a page's `img` (net.rs) and a canvas's images.

use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageFormat, ImageSampler, ImageType};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// An image file's bytes as a picture: any format Bevy's image features decode, as RGBA8 sRGB (float formats taken as
/// linear light and encoded, so bevy_ui samples them filtered).
pub fn decode(bytes: &[u8], mime: &str) -> Option<Image> {
    let fmt = ImageFormat::from_mime_type(mime).or_else(|| {
        let sub = mime.rsplit('/').next()?;
        let sub = sub.strip_prefix("x-").unwrap_or(sub);
        ImageFormat::from_extension(match sub {
            "farbfeld" => "ff",
            "vnd.radiance" => "hdr",
            "vnd-ms.dds" => "dds",
            "targa" => "tga",
            "portable-pixmap" | "portable-anymap" | "portable-graymap" | "portable-bitmap" => "pnm",
            "icon" | "vnd.microsoft.icon" => "ico",
            s => s,
        })
    })?;
    let img = Image::from_buffer(bytes, ImageType::Format(fmt), CompressedImageFormats::NONE, true, ImageSampler::Default, RenderAssetUsages::default()).ok()?;
    let mut img = srgb8(img);
    // bevy_image 0.20 transcodes a DDS's D3DFMT_R8G8B8 (24 bits, red mask 0xff0000: bytes B, G, R) as R, G, B
    let u32_at = |o: usize| bytes.get(o..o + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    if bytes.starts_with(b"DDS ") && u32_at(88) == Some(24) && u32_at(92) == Some(0x00ff_0000) && img.texture_descriptor.format == TextureFormat::Rgba8UnormSrgb {
        if let Some(d) = img.data.as_mut() {
            d.chunks_exact_mut(4).for_each(|p| p.swap(0, 2));
        }
    }
    Some(img)
}

fn srgb8(mut img: Image) -> Image {
    let size = img.size();
    match img.texture_descriptor.format {
        TextureFormat::Rgba8UnormSrgb => img,
        // 8-bit colour stored as unorm (an uncompressed DDS): its bytes are sRGB as a browser takes them
        TextureFormat::Rgba8Unorm => {
            img.texture_descriptor.format = TextureFormat::Rgba8UnormSrgb;
            img
        }
        // 16 bits a channel (farbfeld, 16-bit PNG): the high byte, as sRGB
        TextureFormat::Rgba16Unorm => {
            let px: Vec<u8> = img.data.as_deref().unwrap_or_default().chunks_exact(2).map(|b| b[1]).collect();
            rgba8(size, px)
        }
        TextureFormat::Rgba16Float | TextureFormat::Rgba32Float => {
            let Ok(d) = img.clone().try_into_dynamic() else { return img };
            let f = d.into_rgba32f();
            let enc = |c: f32| {
                let c = c.clamp(0.0, 1.0);
                let s = if c <= 0.003_130_8 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 };
                (s * 255.0 + 0.5) as u8
            };
            let px: Vec<u8> = f.as_raw().chunks_exact(4).flat_map(|p| [enc(p[0]), enc(p[1]), enc(p[2]), (p[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8]).collect();
            rgba8(size, px)
        }
        _ => img.convert(TextureFormat::Rgba8UnormSrgb).unwrap_or(img),
    }
}

fn rgba8(size: UVec2, px: Vec<u8>) -> Image {
    Image::new(Extent3d { width: size.x.max(1), height: size.y.max(1), depth_or_array_layers: 1 }, TextureDimension::D2, px, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_by_media_type() {
        // a 1x1 red PNG
        let png = [
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 0x0d, 0x49, 0x48, 0x44, 0x52, 0, 0, 0, 1, 0, 0, 0, 1, 8, 2, 0, 0, 0, 0x90, 0x77, 0x53, 0xde, 0, 0, 0, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08,
            0xd7, 0x63, 0xf8, 0xcf, 0xc0, 0, 0, 0x03, 0x01, 0x01, 0, 0x18, 0xdd, 0x8d, 0xb0, 0, 0, 0, 0, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];
        let img = decode(&png, "image/png").unwrap();
        assert_eq!(img.size(), UVec2::ONE);
        assert_eq!(img.texture_descriptor.format, TextureFormat::Rgba8UnormSrgb);
        assert!(decode(&png, "image/nope").is_none());
    }
}
