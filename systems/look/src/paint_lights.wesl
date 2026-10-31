// paint_lights.wesl: the point and spot lights that ask for paint (a <PointLight paint> or <SpotLight paint>), in the
// paint look's flat bands (paint.wesl). Each light is read from the fragment's cluster as Bevy's own forward pass reads
// it: how square the surface faces it, its range window and inverse square, a spot's cone, and its shadow when it
// casts one. Where its light on the surface comes to `k.y` lux (a quarter of a DirectionalLight of 1, so a light of
// intensity I reaches about 6 * sqrt(I) metres), the surface takes the light's band, cut hard like the sun's bands and
// ragged by the same `edge` noise: its own colour times the light's colour scaled to a luminance of 1 (a white light:
// the surface's own colour, as the sun lights it; a coloured one as bright, its hue; at most 3 a channel). The band
// takes the place of a shade (`mid`, `shadow`); on a surface a sun lights it adds only what's above the sun's band. At
// `k.z` times the band's lux it steps once more, times `k.w` (brighter with intensity). Lights that didn't ask, and
// every light while none did (`k.x` = 0), change nothing.
import bevy_pbr::render::mesh_view_bindings as view_bindings;
import bevy_pbr::render::mesh_view_types;
import bevy_pbr::render::clustered_forward as clustering;
import bevy_pbr::render::shadows;
import bevy_pbr::render::mesh_types::MESH_FLAGS_SHADOW_RECEIVER_BIT;

struct PaintLights {
    // the asking lights' positions (world, m): a cluster's light is one of them when it sits within a millimetre
    pos: array<vec4<f32>, 8>,
    // how many, the band's lux, the hot band's multiple of it, the hot band's brightness
    k: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(101)
var<uniform> paint_lights: PaintLights;

fn asked(p: vec3<f32>) -> bool {
    let n = u32(paint_lights.k.x);
    for (var j: u32 = 0u; j < n; j = j + 1u) {
        let d = paint_lights.pos[j].xyz - p;
        if dot(d, d) < 1e-6 {
            return true;
        }
    }
    return false;
}

/// the tint a surface's bands give it (`tint`), raised by each asking light that reaches it to that light's band
fn light_bands(tint: vec3<f32>, sunlit: bool, P: vec4<f32>, N: vec3<f32>, world_normal: vec3<f32>, frag_coord: vec4<f32>, view_z: f32, is_orthographic: bool, mesh_flags: u32, wobble: f32, edge: f32) -> vec3<f32> {
    if paint_lights.k.x < 0.5 {
        return tint;
    }
    // the brightest band of the lights that reach it (none: -1)
    var best = vec3<f32>(-1.0);
    let receives = (mesh_flags & MESH_FLAGS_SHADOW_RECEIVER_BIT) != 0u;
    let cluster_index = clustering::view_fragment_cluster_index(frag_coord.xy, view_z, is_orthographic);
    let ranges = clustering::unpack_clusterable_object_index_ranges(cluster_index);
    for (var i: u32 = ranges.first_point_light_index_offset; i < ranges.first_rect_light_index_offset; i = i + 1u) {
        let id = clustering::get_clusterable_object_id(i);
        let light = &view_bindings::clustered_lights.data[id];
        if !asked((*light).position_radius.xyz) {
            continue;
        }
        let to_light = (*light).position_radius.xyz - P.xyz;
        let d2 = dot(to_light, to_light);
        let L = to_light * inverseSqrt(max(d2, 1e-8));
        let facing = dot(N, L) + wobble;
        if facing <= 0.03 {
            continue;
        }
        // range window and inverse square, as Bevy's getDistanceAttenuation; colour_inverse_square_range.rgb is the
        // colour times lumens / 4pi (candela), so this is lux on a surface facing it
        let f = d2 * (*light).color_inverse_square_range.w;
        let window = saturate(1.0 - f * f);
        let rgb = (*light).color_inverse_square_range.rgb;
        let cd = max(rgb.r, max(rgb.g, rgb.b));
        if cd <= 0.0 {
            continue;
        }
        var cone = 1.0;
        if i >= ranges.first_spot_light_index_offset {
            var spot_dir = vec3<f32>((*light).light_custom_data.x, 0.0, (*light).light_custom_data.y);
            spot_dir.y = sqrt(max(0.0, 1.0 - spot_dir.x * spot_dir.x - spot_dir.z * spot_dir.z));
            if ((*light).flags & mesh_view_types::POINT_LIGHT_FLAGS_SPOT_LIGHT_Y_NEGATIVE) != 0u {
                spot_dir.y = -spot_dir.y;
            }
            let a = saturate(dot(-spot_dir, L) * (*light).light_custom_data.z + (*light).light_custom_data.w);
            cone = a * a;
        }
        // the light on it, the band's cut wandering by the edge noise
        let lux = cd * window * window / max(d2, 1e-4) * min(facing, 1.0) * cone * (1.0 + 2.0 * edge);
        if lux < paint_lights.k.y {
            continue;
        }
        if receives && ((*light).flags & mesh_view_types::POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u {
            var s = 1.0;
            if i >= ranges.first_spot_light_index_offset {
                s = shadows::fetch_spot_shadow(id, P, world_normal, (*light).shadow_map_near_z, frag_coord.xy);
            } else {
                s = shadows::fetch_point_shadow(id, P, world_normal, frag_coord.xy);
            }
            // cut at half, as the sun's
            if s < 0.5 + edge {
                continue;
            }
        }
        var band = min(rgb / max(dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722)), 1e-6), vec3<f32>(3.0));
        if lux >= paint_lights.k.y * paint_lights.k.z {
            band = band * paint_lights.k.w;
        }
        best = max(best, band);
    }
    if best.x < 0.0 {
        return tint;
    }
    if sunlit {
        return max(tint, best);
    }
    return best;
}
