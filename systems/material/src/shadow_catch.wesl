// shadow_catch.rs: a surface that shows only the shadows falling on it. Its alpha is its opacity times the share of
// light blocked, every shadow-casting light's shadow multiplied (three.js's ShadowMaterial: getShadowMask).

import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings as view_bindings;
import bevy_pbr::render::mesh_view_types;
import bevy_pbr::render::clustered_forward as clustering;
import bevy_pbr::render::shadows;

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> color: vec4<f32>;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> @location(0) vec4<f32> {
    let P = in.world_position;
    let N = normalize(select(-in.world_normal, in.world_normal, is_front));
    let frag = in.position;
    let view_z = dot(vec4<f32>(
        view_bindings::view.view_from_world[0].z,
        view_bindings::view.view_from_world[1].z,
        view_bindings::view.view_from_world[2].z,
        view_bindings::view.view_from_world[3].z
    ), P);
    let is_orthographic = view_bindings::view.clip_from_view[3].w == 1.0;

    var lit = 1.0;
    for (var i: u32 = 0u; i < view_bindings::lights.n_directional_lights; i = i + 1u) {
        let light = &view_bindings::lights.directional_lights[i];
        if ((*light).flags & mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u {
            lit *= shadows::fetch_directional_shadow(i, P, N, view_z, frag.xy);
        }
    }
    let cluster_index = clustering::view_fragment_cluster_index(frag.xy, view_z, is_orthographic);
    let ranges = clustering::unpack_clusterable_object_index_ranges(cluster_index);
    for (var i: u32 = ranges.first_point_light_index_offset; i < ranges.first_reflection_probe_index_offset; i = i + 1u) {
        let id = clustering::get_clusterable_object_id(i);
        let light = &view_bindings::clustered_lights.data[id];
        if ((*light).flags & mesh_view_types::POINT_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) == 0u {
            continue;
        }
        if i >= ranges.first_spot_light_index_offset {
            lit *= shadows::fetch_spot_shadow(id, P, N, (*light).shadow_map_near_z, frag.xy);
        } else {
            lit *= shadows::fetch_point_shadow(id, P, N, frag.xy);
        }
    }
    return vec4<f32>(color.rgb, color.a * (1.0 - lit));
}
