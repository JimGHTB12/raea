//! A picture three samples without a colour space (NoColorSpace: a distance field, a data map, lettra's MSDF atlas) is
//! read as its raw numbers. Bevy loads a picture as sRGB, so such a texture is a copy of the loaded image with its
//! format's sRGB read dropped (the same bytes), made once a picture once it has loaded.
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Raw {
    pending: Vec<(Handle<Image>, Handle<Image>)>,
    made: HashMap<AssetId<Image>, Handle<Image>>,
}

/// the raw copy of a picture (the same copy each time it's asked for)
pub fn raw(world: &mut World, h: Handle<Image>) -> Handle<Image> {
    if let Some(c) = world.get_resource::<Raw>().and_then(|r| r.made.get(&h.id())) {
        return c.clone();
    }
    let copy = world.resource::<Assets<Image>>().reserve_handle();
    let mut r = world.get_resource_or_insert_with(Raw::default);
    r.made.insert(h.id(), copy.clone());
    r.pending.push((h, copy.clone()));
    copy
}

pub fn sync(r: Option<ResMut<Raw>>, mut images: ResMut<Assets<Image>>) {
    let Some(mut r) = r else { return };
    if r.pending.is_empty() {
        return;
    }
    for (src, copy) in std::mem::take(&mut r.pending) {
        let Some(img) = images.get(&src) else {
            r.pending.push((src, copy));
            continue;
        };
        let mut img = img.clone();
        img.texture_descriptor.format = img.texture_descriptor.format.remove_srgb_suffix();
        if let Some(f) = img.texture_view_descriptor.as_mut().and_then(|v| v.format.as_mut()) {
            *f = f.remove_srgb_suffix();
        }
        let _ = images.insert(copy.id(), img);
    }
}
