//! Screens' pictures from rows (docs/ts-native.feature P27): raea's bevy_ui page draws each `<Screen>`'s nested
//! composition into an image of its own and writes it as an asset, as it does the page; a `screenpage` row says which
//! picture a screen shows over a run of frames ({name, src: "asset:HASH", from, to}). Each frame, before it draws, the
//! picture of every screen whose run starts there is written into the screen's image (screen.rs ScreenImages), so a
//! render from rows shows on its screens what the fed render copied in (Engine::copy_screen) and live play does. The
//! screen's spec (CRT or flat, `pass`, brightness, alpha) is screen.rs's as before; only where the picture comes from
//! differs.
//!
//! The asset PNGs hold straight alpha (react's store_page); screen.wesl reads the page premultiplied in sRGB, as the
//! paint and bevy_ui both make it, so the picture is premultiplied again here.

use super::*;

/// A `screenpage` row: the screen it's for, its picture's file, and the frames [from, to) it shows on.
#[derive(Component)]
pub struct ScreenPage {
    name: String,
    file: PathBuf,
    from: i64,
    to: i64,
}

/// The picture each screen shows now (its file), so an unchanged run isn't written again.
#[derive(Resource, Default)]
struct Shown(HashMap<String, PathBuf>);

/// Pictures handed over by the host for the frame about to draw (a live engine beside the bevy_ui page:
/// Engine::set_screen_picture), straight RGBA by screen name; each written into its screen's image once the screen has
/// one, then dropped.
#[derive(Resource, Default)]
struct Handed(HashMap<String, (u32, u32, Vec<u8>)>);

pub fn install(app: &mut App) {
    app.init_resource::<Shown>()
        .init_resource::<Handed>()
        .add_systems(Update, (show, handed).after(super::sync_screens).before(FrameSystems::Tag));
}

/// Screen `name`'s picture for the next frame drawn (`rgba`: straight, `w` x `h`), as the bevy_ui page drew its nested
/// composition; written into the screen's image as the frame updates (the frame a screen first appears included).
pub fn set_screen_picture(app: &mut App, name: &str, w: u32, h: u32, rgba: Vec<u8>) {
    app.world_mut().resource_mut::<Handed>().0.insert(name.to_string(), (w, h, rgba));
}

/// The handed pictures written into their screens' images (those whose screen has an image yet).
fn handed(mut handed: ResMut<Handed>, screens: Res<ScreenImages>, mut images: ResMut<Assets<Image>>) {
    let names: Vec<String> = handed.0.keys().filter(|n| screens.0.contains_key(*n)).cloned().collect();
    for name in names {
        let Some((pw, ph, px)) = handed.0.remove(&name) else { continue };
        let (handle, w, h) = &screens.0[&name];
        let _ = images.insert(handle.id(), picture(&px, pw, ph, *w, *h));
    }
}

/// A screen's image `w` x `h` from a straight-alpha picture `pw` x `ph`: premultiplied (screen.wesl reads it so), the
/// picture at its top left, clipped; made as screen.rs makes the image (its data in the render world only).
fn picture(px: &[u8], pw: u32, ph: u32, w: u32, h: u32) -> Image {
    let mut data = vec![0u8; (w * h * 4) as usize];
    for y in 0..ph.min(h) {
        for x in 0..pw.min(w) {
            let s = ((y * pw + x) * 4) as usize;
            let a = px[s + 3] as u32;
            let i = ((y * w + x) * 4) as usize;
            for c in 0..3 {
                data[i + c] = ((px[s + c] as u32 * a + 127) / 255) as u8;
            }
            data[i + 3] = px[s + 3];
        }
    }
    let mut img = Image::new(
        bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    );
    img.texture_descriptor.usage |= TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING;
    img
}

/// The row's component, its asset resolved from the edit's folder (`dir`/assets/HASH.png).
pub fn spawn(world: &mut World, e: Entity, t: &Tup, dir: &Path) -> Result<(), String> {
    let v = &t.vals;
    let name = match v.get("name") {
        Some(V::S(s)) => s.clone(),
        _ => return Err(format!("screenpage {}: needs name", t.id)),
    };
    let file = match v.get("src") {
        Some(V::S(s)) => match s.strip_prefix("asset:") {
            Some(h) => dir.join("assets").join(format!("{h}.png")),
            None => dir.join(s),
        },
        _ => return Err(format!("screenpage {}: needs src", t.id)),
    };
    let n = |k: &str| v.get(k).map(V::num).unwrap_or(0.0) as i64;
    world.entity_mut(e).insert(ScreenPage { name, file, from: n("from"), to: n("to") });
    Ok(())
}

/// Each screen's picture for the clock's frame written into its image, where it changed.
fn show(clock: Res<Clock>, pages: Query<&ScreenPage>, screens: Res<ScreenImages>, mut shown: ResMut<Shown>, mut images: ResMut<Assets<Image>>) {
    for p in &pages {
        if clock.0 < p.from || clock.0 >= p.to || shown.0.get(&p.name) == Some(&p.file) {
            continue;
        }
        let Some((handle, w, h)) = screens.0.get(&p.name) else { continue };
        let pic = match image::open(&p.file) {
            Ok(i) => i.to_rgba8(),
            Err(e) => {
                warn!("screen {:?}: its picture {}: {e}", p.name, p.file.display());
                continue;
            }
        };
        let (pw, ph) = pic.dimensions();
        let _ = images.insert(handle.id(), picture(pic.as_raw(), pw, ph, *w, *h));
        shown.0.insert(p.name.clone(), p.file.clone());
    }
}
