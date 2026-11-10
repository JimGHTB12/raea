use super::*;

/// A sprite sheet's cell `frame` of `cols` x `rows` (0 the top left, read along each row) as a card's UV transform.
pub fn sheet_uv(frame: f32, cols: f32, rows: f32) -> bevy::math::Affine2 {
    let (c, r) = (cols.max(1.0), rows.max(1.0));
    let k = frame.max(0.0).floor() % (c * r);
    let (x, y) = (k % c, (k / c).floor());
    bevy::math::Affine2::from_scale_angle_translation(Vec2::new(1.0 / c, 1.0 / r), 0.0, Vec2::new(x / c, y / r))
}

/// An up vector that isn't along `dir` (a light pointing straight down).
pub fn up_for(dir: Vec3) -> Vec3 {
    if dir.normalize_or_zero().cross(Vec3::Y).length_squared() < 1e-6 { Vec3::Z } else { Vec3::Y }
}

/// The camera row's lens and post-processing.
pub fn camera(world: &mut World, e: Entity, vals: &BTreeMap<String, V>) {
    let g = |k: &str, d: f32| num(vals, k, d);
    layers(world, e, vals);
    // its order, clear and viewport among the cameras on its target (<Camera3d order clear_color viewport>)
    raea_sys_frame::camera::camera_pass(world, e, vals);
    // shadow quality (a <Shadows>): the sun's and spots' maps are 4096 texels a side unless it says otherwise (Bevy's
    // 2048 speckles and blurs a film's shadows; smaller on a GPU with little memory of its own, raea_sys_frame::shadow);
    // point lights' cube maps half that, 1024 by default as Bevy's
    if !preview() {
        let size = match vals.get("shadow_map") { Some(V::N(n)) => *n as usize, _ => raea_sys_frame::shadow::shadow_map_default(world) };
        if let Some(mut m) = world.get_resource_mut::<bevy::light::DirectionalLightShadowMap>() {
            m.size = size;
        }
        if let Some(mut m) = world.get_resource_mut::<bevy::light::PointLightShadowMap>() {
            m.size = if vals.contains_key("shadow_map") { size / 2 } else { 1024 };
        }
    }
    let filter = match vals.get("shadow_filter") { Some(V::S(s)) if s == "hardware" => bevy::light::ShadowFilteringMethod::Hardware2x2, _ => bevy::light::ShadowFilteringMethod::Gaussian };
    world.entity_mut(e).insert(filter);
    if vals.contains_key("near") || vals.contains_key("far") {
        let fov = g("fov", 45.0).to_radians();
        world.entity_mut(e).insert(Projection::Perspective(PerspectiveProjection { fov, near: g("near", 0.1), far: g("far", 1000.0), ..default() }));
    }
    // a procedural sky (<Atmosphere>): Bevy 0.20's scattering atmosphere, Earth's air round a planet whose
    // ground is at y 0, lit by the suns; its sky drawn behind everything, and with `atmo_light` it lights the scene too
    // (AtmosphereEnvironmentMapLight). AtmosphereSettings needs Hdr, which it brings
    if vals.get("atmosphere").is_some_and(V::truthy) {
        use bevy::light::{atmosphere::ScatteringMedium, Atmosphere};
        let medium = world.resource_mut::<Assets<ScatteringMedium>>().add(ScatteringMedium::earth(256, 256));
        let mut atmo = Atmosphere::earth(medium);
        if let Some(a) = vals.get("atmo_albedo").map(V::num) {
            atmo.ground_albedo = Vec3::splat(a as f32);
        }
        world.spawn((atmo, super::Mirror));
        world.entity_mut(e).insert(bevy::pbr::AtmosphereSettings::default());
        if vals.get("atmo_light").is_some_and(V::truthy) {
            world.entity_mut(e).insert(bevy::light::AtmosphereEnvironmentMapLight::default());
        }
    }
    let bloom = g("bloom", 0.0);
    if bloom > 0.0 {
        let mut b = bevy::post_process::bloom::Bloom::NATURAL;
        b.intensity = bloom;
        if let Some(th) = vals.get("bloom_threshold").map(V::num) {
            b.prefilter.threshold = th as f32;
            b.prefilter.threshold_softness = 0.2;
        }
        world.entity_mut(e).insert(b);
    }
    if vals.contains_key("fog_color") || vals.contains_key("fog_end") || vals.contains_key("fog_density") {
        let falloff = match vals.get("fog_density") {
            Some(d) => FogFalloff::Exponential { density: d.num() as f32 },
            None => FogFalloff::Linear { start: g("fog_start", 5.0), end: g("fog_end", 20.0) },
        };
        world.entity_mut(e).insert(DistanceFog { color: colour(vals.get("fog_color"), [0.5, 0.5, 0.55, 1.0]), falloff, ..default() });
    }
    if let Some(f) = vals.get("dof_aperture").map(V::num) {
        use bevy::post_process::dof::{DepthOfField, DepthOfFieldMode};
        let mode = if matches!(vals.get("dof_mode"), Some(V::S(m)) if m == "gaussian") { DepthOfFieldMode::Gaussian } else { DepthOfFieldMode::Bokeh };
        // the background counts as far as the camera sees, not infinitely far (its blur would be the largest)
        world.entity_mut(e).insert(DepthOfField { mode, focal_distance: g("focus", 5.0), aperture_f_stops: f as f32, max_circle_of_confusion_diameter: g("dof_max", 64.0), max_depth: g("far", 100.0), ..default() });
    }
    if let Some(V::S(tm)) = vals.get("tonemap") {
        use bevy::core_pipeline::tonemapping::Tonemapping as T;
        let t = match tm.as_str() {
            "none" => T::None,
            "reinhard" => T::Reinhard,
            "aces" => T::AcesFitted,
            "agx" => T::AgX,
            "neutral" | "blender" => T::BlenderFilmic,
            _ => T::TonyMcMapface,
        };
        // "none" draws as Bevy 0.19's None did (bevy_api::Untonemapped)
        if t == T::None {
            world.entity_mut(e).insert((t, super::bevy_api::Untonemapped));
        } else {
            world.entity_mut(e).insert(t).remove::<super::bevy_api::Untonemapped>();
        }
    }
    if let Some(ev) = vals.get("exposure") {
        world.entity_mut(e).insert(bevy::camera::Exposure { ev100: ev.num() as f32 });
    }
    // a <Grade>: Bevy's colour grading (before tone mapping, but for the global saturation), vignette and fringes
    if vals.keys().any(|k| k.starts_with("grade_")) {
        use bevy::render::view::{ColorGrading, ColorGradingSection};
        let mut cg = ColorGrading::default();
        cg.global.temperature = g("grade_temperature", 0.0);
        cg.global.tint = g("grade_tint", 0.0);
        cg.global.post_saturation = g("grade_saturation", 1.0);
        for (pre, sec) in [("grade_s_", &mut cg.shadows), ("grade_m_", &mut cg.midtones), ("grade_h_", &mut cg.highlights)] {
            let d = ColorGradingSection::default();
            *sec = ColorGradingSection {
                saturation: g(&format!("{pre}saturation"), d.saturation),
                contrast: g(&format!("{pre}contrast"), d.contrast),
                gamma: g(&format!("{pre}gamma"), d.gamma),
                gain: g(&format!("{pre}gain"), d.gain),
                lift: g(&format!("{pre}lift"), d.lift),
            };
        }
        world.entity_mut(e).insert(cg);
    }
    if let Some(v) = vals.get("vignette").map(V::num) {
        use bevy::post_process::effect_stack::Vignette;
        let d = Vignette::default();
        world.entity_mut(e).insert(Vignette {
            intensity: v as f32,
            radius: g("vignette_radius", d.radius),
            smoothness: g("vignette_smooth", d.smoothness),
            color: colour(vals.get("vignette_color"), [0.0, 0.0, 0.0, 1.0]),
            ..d
        });
    }
    // ink outlines from the depth and normal prepasses (Msaa::Off, set where the camera is made)
    if vals.contains_key("outline") {
        let line = colour(vals.get("outline"), [0.0, 0.0, 0.0, 1.0]).to_linear().to_f32_array();
        world.entity_mut(e).insert((
            super::outline::InkOutline { color: Vec4::from_array(line), depth: g("outline_depth", 0.04), normal: g("outline_normal", 0.3), width: g("outline_width", 1.0).max(1.0), far: g("outline_far", 0.0) / g("near", 0.1), skip: super::compose::bits(vals, "outline_skip") },
            bevy::core_pipeline::prepass::DepthPrepass,
            bevy::core_pipeline::prepass::NormalPrepass,
        ));
    }
    // ambient occlusion (Msaa::Off, set where the camera is made; its noise is a fixed pattern without TAA's jitter, so
    // a frame's occlusion is a function of its rows) and the anti-aliasing that replaces multisampling
    if let Some(V::S(q)) = vals.get("ssao").filter(|_| !preview()) {
        use bevy::pbr::{ScreenSpaceAmbientOcclusion, ScreenSpaceAmbientOcclusionQualityLevel as Q};
        let quality_level = match q.as_str() { "low" => Q::Low, "high" => Q::High, "ultra" => Q::Ultra, _ => Q::Medium };
        world.entity_mut(e).insert(ScreenSpaceAmbientOcclusion {
            quality_level,
            constant_object_thickness: g("ssao_thickness", 0.25),
            // the radius is Bevy's default (0.20's is 0.19's hard-coded 0.5 x 1.457)
            ..default()
        });
    }
    match vals.get("aa") {
        Some(V::S(m)) if m == "smaa" && !preview() => {
            world.entity_mut(e).insert(bevy::anti_alias::smaa::Smaa { preset: bevy::anti_alias::smaa::SmaaPreset::High });
        }
        Some(V::S(m)) if m == "fxaa" && !preview() => {
            world.entity_mut(e).insert(bevy::anti_alias::fxaa::Fxaa::default());
        }
        _ => {}
    }
    // volumetric fog: light in the air inside each fog volume. Its jitter is raea's dither (install_fog_shader): 0
    // here, and with `vfog_dither` set each frame by sync_fog_dither from the film frame, so a frame's fog is a
    // function of the frame either way
    // (none when drawing a pass: passes.rs)
    if vals.get("vfog").is_some_and(V::truthy) && super::passes::wanted().is_none() {
        // RAEA_FOG_REFERENCE=1: the fog's ground truth (sdk/raea/bench/bevy/fog-score.py scores against it): Bevy's
        // march at 512 steps (RAEA_FOG_REFERENCE_STEPS), undithered, at full resolution, whatever the composition asks
        // for. Converged by then (the rome curia: 256 against 512 SSIM 0.99988); 1024 at 1080p runs past what Metal
        // lets one pass take and the fog pass is dropped without an error, the frame drawn with no fog at all
        let reference = fog_reference();
        let fog = bevy::light::VolumetricFog {
            step_count: if reference { std::env::var("RAEA_FOG_REFERENCE_STEPS").ok().and_then(|v| v.parse().ok()).unwrap_or(512) } else { g("vfog_steps", 64.0).round().max(1.0) as u32 },
            ambient_intensity: g("vfog_ambient", 0.0),
            ambient_color: colour(vals.get("vfog_ambient_color"), [1.0, 1.0, 1.0, 1.0]),
            jitter: 0.0,
        };
        // every volume in one grid along the view (froxel.rs), unless vfog_mode is march (or RAEA_FOG_MODE=march, for
        // comparing): Bevy's march, where vfog_resolution below 1 marches at that fraction of the frame, upsampled by
        // depth (fog_half.rs). The grid scored at or above the march on every scene fog-score.py judges (the
        // fixtures, the rome film's plot and curia, Mean Streets), in SSIM, PSNR, ms a frame and bitrate, so it's the
        // default; RAEA_FOG_MODE=froxel forces it over a march row
        let env_mode = std::env::var("RAEA_FOG_MODE");
        let march = vals.get("vfog_mode").is_some_and(|m| matches!(m, V::S(s) if s == "march")) && env_mode.as_deref() != Ok("froxel");
        let froxel = !reference && !march && env_mode.as_deref() != Ok("march");
        if froxel {
            let lin = fog.ambient_color.to_linear();
            super::froxel::insert(
                world,
                e,
                super::froxel::FroxelFog {
                    ambient: Vec3::new(lin.red, lin.green, lin.blue) * fog.ambient_intensity,
                    slices: g("vfog_slices", 64.0).round().clamp(32.0, 256.0) as u32,
                    tile: g("vfog_tile", 16.0).round().clamp(4.0, 16.0) as u32,
                    samples: g("vfog_samples", 1.0).round().clamp(1.0, 8.0) as u32,
                    // (the ground truth integrates every light itself)
                    hero: g("vfog_hero", 1.0) != 0.0 && std::env::var("RAEA_FOG_HERO").as_deref() != Ok("0") && super::froxel::reference_steps().is_none(),
                },
            );
        } else {
            super::fog_half::insert(world, e, fog, if reference { 1.0 } else { g("vfog_resolution", 1.0) });
            if !reference && vals.get("vfog_dither").is_some_and(V::truthy) {
                world.entity_mut(e).insert(FogDither);
            }
        }
    }
    if let Some(a) = vals.get("aberration").map(V::num) {
        use bevy::post_process::effect_stack::ChromaticAberration;
        // its intensity is each frame's (sync_camera)
        world.entity_mut(e).insert(ChromaticAberration { intensity: a as f32, ..default() });
    }
    if super::lens::has_lens(vals) {
        // Fisheye / LensDistortion: its values come from each frame's row (sync_camera)
        world.entity_mut(e).insert(super::lens::Lens::default());
    }
}

/// Whether RAEA_FOG_REFERENCE asks for the fog's ground truth instead of what the composition asks for.
pub fn fog_reference() -> bool {
    std::env::var("RAEA_FOG_REFERENCE").is_ok_and(|v| v == "1")
}

/// raea's volumetric fog shader (volumetric_fog.wesl: Bevy 0.20.0's, its dither seeded from the film frame) over
/// Bevy's embedded one. The fog pipeline loads its shader when it is first made, after the plugins are built, so the
/// copy registered here is the one it gets.
pub fn install_fog_shader(app: &mut App) {
    let swapped = super::bevy_api::swap_embedded_shader(app, "bevy_pbr/volumetric_fog/volumetric_fog.wesl", 0xd6b9_2688_4641_46be, |_| {
        let mut s = include_str!("volumetric_fog.wesl").to_string();
        // the reference marches the whole ray (Bevy's stops short off the view axis) and scatters the ambient light
        // by density, as the grid does (RAEA_FOG_REFERENCE_AMBIENT=flat: Bevy's flat ambient)
        if fog_reference() {
            s = replace_once(&s, "const RAY_LENGTH: bool = false;", "const RAY_LENGTH: bool = true;");
            if std::env::var("RAEA_FOG_REFERENCE_AMBIENT").as_deref() != Ok("flat") {
                s = replace_once(&s, "const SCATTERED_AMBIENT: bool = false;", "const SCATTERED_AMBIENT: bool = true;");
            }
        }
        s
    });
    assert!(swapped, "raea: Bevy no longer embeds bevy_pbr/volumetric_fog/volumetric_fog.wesl (docs/bevy-upgrade.md)");
}

/// A camera whose volumetric fog is dithered (`<VolumetricFog dither>`).
#[derive(Component)]
pub struct FogDither;

/// The dither's seed for this frame: Bevy's `jitter`, which raea's shader reads as 1 + the film frame mod 64 (0 is no
/// dither). The film frame, not the engine's update count, so a frame rendered alone and the same frame reached in
/// play march the same samples.
pub fn sync_fog_dither(
    clock: Res<super::Clock>,
    mut q: Query<&mut bevy::light::VolumetricFog, With<FogDither>>,
    mut half: Query<&mut super::fog_half::HalfFog, With<FogDither>>,
) {
    let seed = 1.0 + clock.0.rem_euclid(64) as f32;
    for mut f in &mut q {
        if f.jitter != seed {
            f.jitter = seed;
        }
    }
    for mut f in &mut half {
        if f.fog.jitter != seed {
            f.fog.jitter = seed;
        }
    }
}
