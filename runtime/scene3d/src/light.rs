use super::*;
use super::froxel::{FogRank, FogThinning};

/// Whether a light casts shadows when it's lit (its row's `shadows`, or `volumetric`), kept apart from its
/// `shadow_maps_enabled`, which sync_lights clears while the light is dark.
#[derive(Component)]
pub struct CastsShadows(pub bool);

/// A light row: directional (the default, as before), point or spot.
pub fn light(world: &mut World, e: Entity, vals: &BTreeMap<String, V>) {
    let g = |k: &str, d: f32| num(vals, k, d);
    layers(world, e, vals);
    let color = colour(vals.get("color"), [1.0, 1.0, 1.0, 1.0]);
    let kind = match vals.get("type") { Some(V::S(s)) => s.as_str(), _ => "directional" };
    let pos = Vec3::new(g("x", 0.0), g("y", 0.0), g("z", 0.0));
    let dir = Vec3::new(g("dir_x", -0.5), g("dir_y", -1.0), g("dir_z", -0.4));
    // a light seen in the air (volumetric) is marched through its shadow map, so it casts shadows
    let volumetric = flag(vals, "volumetric", false) && !preview();
    // shadows by default for the sun, not for point or spot lights; sync_lights turns them off while the light is dark
    let directional = kind != "point" && kind != "spot" && kind != "rect";
    world.entity_mut(e).insert(CastsShadows((flag(vals, "shadows", directional) || volumetric) && !preview()));
    // soft shadows (PCSS): the light's size; a spot's or bulb's is its radius
    let soft = g("soft", 0.0);
    let radius = if soft > 0.0 { soft } else { g("radius", 0.0) };
    match kind {
        "point" => {
            world.entity_mut(e).insert((
                PointLight {
                    color,
                    intensity: g("lumens", 1.0e6),
                    range: g("range", 20.0),
                    radius,
                    shadow_maps_enabled: (flag(vals, "shadows", false) || volumetric) && !preview(),
                    #[cfg(feature = "pcss")]
                    soft_shadows_enabled: soft > 0.0,
                    ..default()
                },
                Transform::from_translation(pos),
            ));
        }
        // a rectangle of light (<RectAreaLight>, Bevy's RectLight: its LTC lookup tables need area_light_luts): its
        // face, width x height, looking along its direction; it lights surfaces but casts no shadow (Bevy 0.20)
        "rect" => {
            world.entity_mut(e).insert((
                bevy::light::RectLight { color, intensity: g("lumens", 1.0e6), range: g("range", 20.0), width: g("width", 1.0), height: g("height", 1.0) },
                Transform::from_translation(pos).looking_to(dir, up_for(dir)),
            ));
        }
        "spot" => {
            let outer = g("outer", std::f32::consts::FRAC_PI_4);
            world.entity_mut(e).insert((
                SpotLight {
                    color,
                    intensity: g("lumens", 1.0e6),
                    range: g("range", 20.0),
                    radius,
                    #[cfg(feature = "pcss")]
                    soft_shadows_enabled: soft > 0.0,
                    outer_angle: outer,
                    inner_angle: g("inner", 0.0).min(outer),
                    shadow_maps_enabled: (flag(vals, "shadows", false) || volumetric) && !preview(),
                    ..default()
                },
                Transform::from_translation(pos).looking_to(dir, up_for(dir)),
            ));
        }
        _ => {
            // cascades fitted to a film's scale, not Bevy's kilometre: the first ends at 8 m, the last at 150 m, and
            // neighbours overlap by 0.3 so the seam between them doesn't show (a <Shadows> changes them)
            let cascades = bevy::light::CascadeShadowConfigBuilder {
                num_cascades: g("shadow_cascades", 4.0).clamp(1.0, 4.0) as usize,
                minimum_distance: 0.1,
                maximum_distance: g("shadow_dist", 150.0),
                first_cascade_far_bound: g("shadow_first", 8.0).min(g("shadow_dist", 150.0)),
                overlap_proportion: g("shadow_overlap", 0.3),
            }
            .build();
            world.entity_mut(e).insert((
                DirectionalLight {
                    color,
                    illuminance: g("lux", 8000.0),
                    shadow_maps_enabled: (flag(vals, "shadows", true) || volumetric) && !preview(),
                    #[cfg(feature = "pcss")]
                    soft_shadow_size: (soft > 0.0).then_some(soft),
                    ..default()
                },
                cascades,
                Transform::default().looking_to(dir, up_for(dir)),
            ));
        }
    }
    // a <Shadows>' biases, on every light (Bevy's defaults otherwise: 0.02 / 1.8 for the sun and spots, 0.08 / 0.6
    // for point lights)
    let (b, nb) = (vals.get("shadow_bias").map(|v| v.num() as f32), vals.get("shadow_nbias").map(|v| v.num() as f32));
    if b.is_some() || nb.is_some() {
        let mut em = world.entity_mut(e);
        if let Some(mut l) = em.get_mut::<DirectionalLight>() {
            l.shadow_depth_bias = b.unwrap_or(l.shadow_depth_bias);
            l.shadow_normal_bias = nb.unwrap_or(l.shadow_normal_bias);
        }
        if let Some(mut l) = em.get_mut::<SpotLight>() {
            l.shadow_depth_bias = b.unwrap_or(l.shadow_depth_bias);
            l.shadow_normal_bias = nb.unwrap_or(l.shadow_normal_bias);
        }
        if let Some(mut l) = em.get_mut::<PointLight>() {
            l.shadow_depth_bias = b.unwrap_or(l.shadow_depth_bias);
            l.shadow_normal_bias = nb.unwrap_or(l.shadow_normal_bias);
        }
    }
    // soft shadows (PCSS): a sun's size as seen from the scene, or a point or spot light's radius; the shadow's
    // penumbra widens with the distance from what casts it, as a real one's does. Native only (the pcss feature):
    // Bevy 0.20's PCSS samples depth maps with a filtering sampler, which WebGPU refuses, so a browser's shadows are
    // filtered (PCF) without the penumbra
    #[cfg(feature = "pcss")]
    if let Some(soft) = vals.get("soft").map(|v| v.num() as f32).filter(|s| *s > 0.0 && !preview()) {
        let mut em = world.entity_mut(e);
        if let Some(mut l) = em.get_mut::<DirectionalLight>() {
            l.soft_shadow_size = Some(soft);
        }
        if let Some(mut l) = em.get_mut::<SpotLight>() {
            l.radius = soft;
            l.soft_shadows_enabled = true;
        }
        if let Some(mut l) = em.get_mut::<PointLight>() {
            l.radius = soft;
            l.soft_shadows_enabled = true;
        }
    }
    if volumetric {
        world.entity_mut(e).insert(bevy::light::VolumetricLight);
    }
}

/// A fog volume row: a unit box of air (Bevy's FogVolume), placed and sized by its transform.
pub fn fog_volume(world: &mut World, e: Entity, id: &str, vals: &BTreeMap<String, V>) {
    let mut fv = bevy::light::FogVolume::default();
    fog_values(&mut fv, vals);
    world.entity_mut(e).insert((fv, fog_transform(vals), FogRank(id.to_string()), FogThinning(num(vals, "falloff", 0.0).max(0.0))));
}

/// The render world's fog volumes, by rank (filled each extraction).
#[derive(Resource, Default)]
pub struct FogOrder(Vec<(String, Entity)>);

pub fn extract_fog_order(q: bevy::render::Extract<Query<(&bevy::render::sync_world::RenderEntity, &FogRank)>>, mut order: ResMut<FogOrder>) {
    order.0.clear();
    order.0.extend(q.iter().map(|(r, k)| (k.0.clone(), r.id())));
}

/// Takes every fog volume's FogVolume off its render entity and puts them back in rank order, so their table (the
/// order Bevy's prepare_volumetric_fog_uniforms draws them in) is in rank order. Runs before RenderSystems::Prepare.
pub fn order_fog_volumes(world: &mut World) {
    let mut order = std::mem::take(&mut world.resource_mut::<FogOrder>().0);
    if order.len() < 2 {
        return;
    }
    order.sort();
    let mut taken = Vec::with_capacity(order.len());
    for (_, e) in &order {
        if let Ok(mut em) = world.get_entity_mut(*e) {
            if let Some(f) = em.take::<bevy::light::FogVolume>() {
                taken.push((*e, f));
            }
        }
    }
    for (e, f) in taken {
        world.entity_mut(e).insert(f);
    }
}

/// A fog volume's transform, shrunk to nothing while it's hidden: Bevy draws every FogVolume whatever its Visibility,
/// and lights the whole box with the camera's ambient whatever its density.
pub fn fog_transform(v: &BTreeMap<String, V>) -> Transform {
    let t = transform_of(v);
    if v.get("visible").is_none_or(V::truthy) { t } else { t.with_scale(Vec3::ZERO) }
}

/// A fog volume's surface values from its row: density (0 while hidden), colour, absorption, scattering, anisotropy.
pub fn fog_values(fv: &mut bevy::light::FogVolume, v: &BTreeMap<String, V>) {
    let shown = v.get("visible").is_none_or(V::truthy);
    fv.density_factor = if shown { num(v, "density", 0.1).max(0.0) } else { 0.0 };
    fv.fog_color = if v.contains_key("col_r") {
        Color::srgb(num(v, "col_r", 1.0), num(v, "col_g", 1.0), num(v, "col_b", 1.0))
    } else {
        colour(v.get("color"), [1.0, 1.0, 1.0, 1.0])
    };
    fv.absorption = num(v, "absorption", 0.3);
    fv.scattering = num(v, "scattering", 0.3);
    fv.scattering_asymmetry = num(v, "anisotropy", 0.5).clamp(-0.99, 0.99);
}

/// Fog volumes where the frame's rows put them, as thick and as coloured as they say.
pub fn sync_fog_volumes(frame: Res<FrameRows>, mut q: Query<(Entity, &mut Transform, &mut bevy::light::FogVolume)>) {
    for (e, mut t, mut fv) in &mut q {
        let Some(v) = frame.vals(e) else { continue };
        let v = &v;
        let nt = fog_transform(v);
        if *t != nt {
            *t = nt;
        }
        let mut next = fv.clone();
        fog_values(&mut next, v);
        if next.density_factor != fv.density_factor || next.fog_color != fv.fog_color || next.absorption != fv.absorption || next.scattering != fv.scattering || next.scattering_asymmetry != fv.scattering_asymmetry {
            *fv = next;
        }
    }
}
