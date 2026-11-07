//! A raea-render mesh row's `three` column (R3F's reader writes it, docs/r3f.feature R3F11): its material with its
//! pictures found, and the mesh given what the shader reads.

use super::*;

/// The row's three.js shader as its material and the names of the mesh's own attributes it reads. One that can't run
/// here is named in a warning and None, so the plain surface its row also describes is drawn. A picture made in code (a
/// DataTexture, a canvas: R3F's reader sends it as a PNG data: URL) is decoded here as a material's map is; any other
/// is `image`'s to find (a file, a URL, a render: ref).
pub fn of_row(world: &mut World, id: &str, spec: &str, mut image: impl FnMut(&mut World, &Value) -> Result<Handle<Image>, String>) -> Option<(ThreeShader, Vec<String>)> {
    let built = serde_json::from_str::<Value>(spec).map_err(|e| e.to_string()).and_then(|v| {
        build(world, &v, |w, x| match x.as_str() {
            Some(p) if p.starts_with("data:") => {
                let pic = raea_sys_material::maps::data_picture(p)?;
                let img = Image::from_dynamic(image::DynamicImage::ImageRgba8(pic), true, bevy::asset::RenderAssetUsages::default());
                Ok(w.resource_mut::<Assets<Image>>().add(img))
            }
            _ => image(w, x),
        })
    });
    built.map_err(|err| warn!("mesh {id}: its three.js shader doesn't run here, so it's drawn in its colour: {err}")).ok()
}

/// The row's mesh (`base`) given what its shader reads (fit_mesh): a colour and uvs on every vertex, its own attributes.
pub fn fitted(base: Option<&Mesh>, id: &str, buffer: Option<&str>, own: &[String]) -> Result<Mesh, String> {
    let mut m = base.cloned().ok_or_else(|| format!("mesh {id}: its mesh isn't made"))?;
    fit_mesh(&mut m, buffer, own).map_err(|e| format!("mesh {id}: {e}"))?;
    Ok(m)
}
