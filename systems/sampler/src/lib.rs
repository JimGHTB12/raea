//! How a texture is read, given from TypeScript (docs/shaders.feature SH5): `sampler: { filter, mipmap, wrap, anisotropy }`
//! on a material's maps, a shader's textures and a Model's own glTF textures, each part optional, over the texture's
//! default. Bevy keeps the sampler on the Image, so a map made here (maps.rs) is one image per sampler, while an image
//! loaded from a file (a shader's texture by path, a glTF's) gets its sampler once it has loaded (sync_samplers); one
//! file given two samplers is drawn with the last, and the load says so.
//!
//!   filter      "nearest" | "linear"                  magnification and minification ("nearest": hard pixels)
//!   mipmap      "nearest" | "linear" | "none"         between mip levels ("none": the full-size texture only)
//!   wrap        "repeat" | "clamp" | "mirror", or [u, v]
//!   anisotropy  1 to 16 (needs every filter linear; nearest turns it off)

pub mod arrays;

use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;
use raea_core_program::ir::V;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use serde_json::Value;

/// A sampler as given: each part, or the texture's default.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sampling {
    filter: Option<ImageFilterMode>,
    mipmap: Option<Option<ImageFilterMode>>,
    wrap: Option<(ImageAddressMode, ImageAddressMode)>,
    anisotropy: Option<u16>,
    /// as given, for a cache key
    pub key: String,
}

fn filter(s: &str, what: &str) -> Result<ImageFilterMode, String> {
    match s {
        "nearest" => Ok(ImageFilterMode::Nearest),
        "linear" => Ok(ImageFilterMode::Linear),
        _ => Err(format!("a sampler's {what} is \"nearest\" or \"linear\", got {s:?}")),
    }
}

fn wrap(v: &Value) -> Result<ImageAddressMode, String> {
    match v.as_str() {
        Some("repeat") => Ok(ImageAddressMode::Repeat),
        Some("clamp") => Ok(ImageAddressMode::ClampToEdge),
        Some("mirror") => Ok(ImageAddressMode::MirrorRepeat),
        _ => Err(format!("a sampler's wrap is \"repeat\", \"clamp\" or \"mirror\" (or [u, v] of them), got {v}")),
    }
}

impl Sampling {
    pub fn of(v: &Value) -> Result<Sampling, String> {
        let o = v.as_object().ok_or_else(|| format!("a sampler is {{filter, mipmap, wrap, anisotropy}}, got {v}"))?;
        if let Some(k) = o.keys().find(|k| !["filter", "mipmap", "wrap", "anisotropy"].contains(&k.as_str())) {
            return Err(format!("a sampler has filter, mipmap, wrap and anisotropy, not {k:?}"));
        }
        let mut s = Sampling { key: v.to_string(), ..Default::default() };
        if let Some(f) = o.get("filter") {
            s.filter = Some(filter(f.as_str().unwrap_or(""), "filter")?);
        }
        if let Some(m) = o.get("mipmap") {
            s.mipmap = Some(match m.as_str() {
                Some("none") => None,
                Some(x) => Some(filter(x, "mipmap")?),
                None => return Err(format!("a sampler's mipmap is \"nearest\", \"linear\" or \"none\", got {m}")),
            });
        }
        if let Some(w) = o.get("wrap") {
            s.wrap = Some(match w {
                Value::Array(a) if a.len() == 2 => (wrap(&a[0])?, wrap(&a[1])?),
                _ => {
                    let m = wrap(w)?;
                    (m, m)
                }
            });
        }
        if let Some(a) = o.get("anisotropy") {
            let n = a.as_u64().filter(|n| (1..=16).contains(n)).ok_or_else(|| format!("a sampler's anisotropy is a whole number from 1 to 16, got {a}"))?;
            s.anisotropy = Some(n as u16);
        }
        Ok(s)
    }

    /// From a row's JSON column, if it has one.
    pub fn col(vals: &BTreeMap<String, V>, k: &str) -> Result<Option<Sampling>, String> {
        let Some(V::S(s)) = vals.get(k) else { return Ok(None) };
        let v: Value = serde_json::from_str(s).map_err(|e| format!("{k}: {e}"))?;
        Sampling::of(&v).map(Some)
    }

    /// The texture's default sampler with what this gives over it. Anisotropy needs every filter linear (wgpu), so a
    /// nearest filter turns it off; "none" for mipmaps reads the full-size level only.
    pub fn over(&self, mut d: ImageSamplerDescriptor) -> ImageSamplerDescriptor {
        if let Some(f) = self.filter {
            d.mag_filter = f;
            d.min_filter = f;
        }
        match self.mipmap {
            Some(Some(f)) => d.mipmap_filter = f,
            Some(None) => {
                d.mipmap_filter = ImageFilterMode::Nearest;
                d.lod_max_clamp = 0.0;
            }
            None => {}
        }
        if let Some((u, v)) = self.wrap {
            d.address_mode_u = u;
            d.address_mode_v = v;
        }
        if let Some(a) = self.anisotropy {
            d.anisotropy_clamp = a;
        }
        let linear = d.mag_filter == ImageFilterMode::Linear && d.min_filter == ImageFilterMode::Linear && d.mipmap_filter == ImageFilterMode::Linear;
        if !linear {
            d.anisotropy_clamp = 1;
        }
        d
    }
}

/// An image's own sampler as a descriptor (Bevy's default reads as linear, the app's default).
pub fn descriptor_of(s: &ImageSampler) -> ImageSamplerDescriptor {
    match s {
        ImageSampler::Descriptor(d) => d.clone(),
        ImageSampler::Default => ImageSamplerDescriptor::linear(),
    }
}

/// Samplers for images that load from files, set once each has loaded; `set` remembers what each got, so a second,
/// different one for the same file is reported.
#[derive(Resource, Default)]
pub struct PendingSamplers {
    pending: Vec<(Handle<Image>, Sampling, String)>,
    set: HashMap<AssetId<Image>, (Sampling, String)>,
}

impl PendingSamplers {
    /// `who` names the texture for the report (a mesh's map, a shader's texture, a model's file).
    pub fn give(world: &mut World, h: Handle<Image>, s: Sampling, who: String) {
        world.get_resource_or_insert_with(PendingSamplers::default).pending.push((h, s, who));
    }
}

/// A shader's texture as given: a picture (`image` resolves it as a Handle<Image> field does), or `{src, sampler}`; an
/// inline picture may carry its `sampler` too. The sampler is set once the image has loaded.
pub fn texture(world: &mut World, v: &Value, who: &str, image: &mut impl FnMut(&mut World, &Value) -> Result<Handle<Image>, String>) -> Result<Handle<Image>, String> {
    let (pic, sampler) = match v {
        Value::Object(o) if o.contains_key("src") => (o.get("src").cloned().unwrap_or(Value::Null), o.get("sampler")),
        Value::Object(o) => (v.clone(), o.get("sampler")),
        _ => (v.clone(), None),
    };
    let h = image(world, &pic)?;
    if let Some(s) = sampler {
        PendingSamplers::give(world, h.clone(), Sampling::of(s)?, who.to_string());
    }
    Ok(h)
}

pub fn sync_samplers(p: Option<ResMut<PendingSamplers>>, mut images: ResMut<Assets<Image>>) {
    let Some(mut p) = p else { return };
    if p.pending.is_empty() {
        return;
    }
    let pending = std::mem::take(&mut p.pending);
    for (h, s, who) in pending {
        let Some(cur) = images.get(&h).map(|i| descriptor_of(&i.sampler)) else {
            p.pending.push((h, s, who));
            continue;
        };
        if let Some((was, by)) = p.set.get(&h.id()) {
            if *was != s {
                eprintln!("raea: {who}'s sampler {} replaces {by}'s {} on the same image (an image has one sampler: the last given is drawn)", s.key, was.key);
            }
        }
        let want = s.over(cur);
        if let Some(mut img) = images.get_mut(&h) {
            img.sampler = ImageSampler::Descriptor(want);
        }
        p.set.insert(h.id(), (s, who));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_is_hard_and_turns_anisotropy_off() {
        let s = Sampling::of(&serde_json::json!({"filter": "nearest", "wrap": ["clamp", "mirror"]})).unwrap();
        let d = s.over(ImageSamplerDescriptor { anisotropy_clamp: 16, ..ImageSamplerDescriptor::linear() });
        assert_eq!((d.mag_filter, d.min_filter), (ImageFilterMode::Nearest, ImageFilterMode::Nearest));
        assert_eq!((d.address_mode_u, d.address_mode_v), (ImageAddressMode::ClampToEdge, ImageAddressMode::MirrorRepeat));
        assert_eq!(d.anisotropy_clamp, 1);
    }

    #[test]
    fn parts_not_given_keep_the_default() {
        let s = Sampling::of(&serde_json::json!({"mipmap": "none"})).unwrap();
        let d = s.over(ImageSamplerDescriptor { anisotropy_clamp: 16, ..ImageSamplerDescriptor::linear() });
        assert_eq!(d.mag_filter, ImageFilterMode::Linear);
        assert_eq!(d.lod_max_clamp, 0.0);
        assert!(Sampling::of(&serde_json::json!({"filtre": "nearest"})).is_err());
        assert!(Sampling::of(&serde_json::json!({"anisotropy": 32})).is_err());
    }
}
