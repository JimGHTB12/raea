// paint.rs: the paint look's vertex stage, for the main pass and the prepass (depth, normals, shadow maps) alike.
// Its own file: paint.wesl's fragment code can't compile in every prepass. The Paint struct is paint.wesl's (keep the
// two the same).
import bevy_pbr::render::{
    mesh_bindings::mesh,
    mesh_functions,
    skinning,
    morph,
    morph::{morph_position, morph_normal, morph_tangent},
    view_transformations::position_world_to_clip,
};
@if(PREPASS_PIPELINE) import bevy_pbr::prepass::io::{Vertex, UncompressedVertex, VertexOutput, decompress_vertex};
@if(!PREPASS_PIPELINE) import bevy_pbr::render::forward_io::{Vertex, UncompressedVertex, VertexOutput, decompress_vertex};

struct Paint {
    shadow: vec4<f32>,
    mid: vec4<f32>,
    ao: vec4<f32>,
    foam: vec4<f32>,
    patch_: vec4<f32>,
    // bands, ao height, ground y, water y
    k0: vec4<f32>,
    // foam height, ssao threshold, shadow-edge raggedness, breakup amount
    k1: vec4<f32>,
    // breakup scale, edge scale, wash, wash patch size (uv)
    k2: vec4<f32>,
    // stroke amount, stroke length, saturation, foliage
    k3: vec4<f32>,
    // foliage height, calm from, calm to, calm amount
    k4: vec4<f32>,
    calm: vec4<f32>,
    // texel (m: surfaces as pixel art, 0 for none), each cell's shade jitter, cloud shadows' drift (x, z, m)
    k5: vec4<f32>,
    // cloud shadows: cover (0 none to 1), size (m); this material sways in the wind (1: a cut-out leaf or blade), -
    k6: vec4<f32>,
    // wind: how far a blade's tip bends (m, 0 none), its direction (x, z), the time it has blown (s)
    k7: vec4<f32>,
    // haze: how thick (per metre), how fast it thins with height (m), where it starts (m), its most (0 to 1); its
    // colour (a: on), and its colour toward the sun (a: on)
    k8: vec4<f32>,
    haze: vec4<f32>,
    haze_sun: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(100)
var<uniform> paint: Paint;

// The wind (Bevy 0.20's mesh vertex, and its prepass's, with one change): a cut-out blade or leaf near the ground is
// bent downwind by how high it stands (by the square of its height over twice `foliage height`, so the root stays
// put), in gusts that travel across the field with the wind and a quicker flutter of its own. The prepass (depth,
// normals, shadows) bends it the same, so the outlines and cast shadows move with the grass.
fn wind(p: vec3<f32>) -> vec3<f32> {
    let strength = paint.k7.x;
    if strength <= 0.0 || paint.k6.z < 0.5 {
        return p;
    }
    let fh = max(paint.k4.x, 0.01);
    let y = p.y - paint.k0.z;
    let h = clamp(y / (2.0 * fh), 0.0, 1.0);
    let bend = h * h * (1.0 - smoothstep(2.0 * fh, 3.5 * fh, y));
    if bend <= 0.0 {
        return p;
    }
    let dir = normalize(vec2<f32>(paint.k7.y, paint.k7.z) + vec2<f32>(1e-5, 0.0));
    let t = paint.k7.w;
    let along = dot(p.xz, dir);
    // gusts: broad bands rolling downwind, then a slower swell under them; flutter: each clump on its own beat
    let gust = 0.55 + 0.3 * sin(along * 0.45 - t * 2.1) + 0.15 * sin(along * 0.13 + p.x * 0.07 - t * 0.7);
    let flutter = 0.18 * sin(t * 6.3 + p.x * 3.7 + p.z * 2.9) + 0.1 * sin(t * 9.1 + p.z * 5.3);
    let push = strength * bend * (gust + flutter);
    let side = vec2<f32>(-dir.y, dir.x) * strength * bend * 0.25 * sin(t * 4.2 + along * 1.9);
    // the tip leans over rather than stretching: it drops a little as it bends
    return vec3<f32>(p.x + dir.x * push + side.x, p.y - 0.5 * push * push / max(2.0 * fh, 0.01), p.z + dir.y * push + side.y);
}

@if(MORPH_TARGETS)
fn morph_vertex(vertex_in: UncompressedVertex, instance_index: u32) -> UncompressedVertex {
    var vertex = vertex_in;
    let first_vertex = mesh[instance_index].first_vertex_index;
    let vertex_index = vertex.index - first_vertex;
    let morph_descriptor_index = mesh[instance_index].morph_descriptor_index;
    let weight_count = morph::layer_count(morph_descriptor_index);
    for (var i: u32 = 0u; i < weight_count; i ++) {
        let weight = morph::weight_at(i, morph_descriptor_index);
        if weight == 0.0 {
            continue;
        }
        vertex.position += weight * morph_position(vertex_index, i, morph_descriptor_index);
        @if(VERTEX_NORMALS)
        vertex.normal += weight * morph_normal(vertex_index, i, morph_descriptor_index);
        @if(VERTEX_TANGENTS)
        vertex.tangent += vec4(weight * morph_tangent(vertex_index, i, morph_descriptor_index), 0.0);
    }
    return vertex;
}

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let uncompressed_vertex_no_morph = decompress_vertex(vertex_no_morph, vertex_no_morph.instance_index);
    @if(MORPH_TARGETS)
    var vertex = morph_vertex(uncompressed_vertex_no_morph, vertex_no_morph.instance_index);
    @else
    var vertex = uncompressed_vertex_no_morph;
    let mesh_world_from_local = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);
    @if(SKINNED)
    var world_from_local = skinning::skin_model(vertex.joint_indices, vertex.joint_weights, vertex_no_morph.instance_index);
    @else
    var world_from_local = mesh_world_from_local;
    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    out.world_position = vec4<f32>(wind(out.world_position.xyz), 1.0);
    out.position = position_world_to_clip(out.world_position.xyz);
    @if(PREPASS_PIPELINE && UNCLIPPED_DEPTH_ORTHO_EMULATION) {
        out.unclipped_depth = out.position.z;
        out.position.z = min(out.position.z, 1.0);
    }
    @if(VERTEX_UVS_A)
    out.uv = vertex.uv;
    @if(VERTEX_UVS_B)
    out.uv_b = vertex.uv_b;
    // normals and tangents: the main pass always, the prepass only when it writes normals (or is deferred)
    @if(!PREPASS_PIPELINE || NORMAL_PREPASS_OR_DEFERRED_PREPASS) {
        @if(VERTEX_NORMALS && SKINNED)
        out.world_normal = skinning::skin_normals(world_from_local, vertex.normal);
        @elif(VERTEX_NORMALS)
        out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal, vertex_no_morph.instance_index);
        @if(VERTEX_TANGENTS)
        out.world_tangent = mesh_functions::mesh_tangent_local_to_world(world_from_local, vertex.tangent, vertex_no_morph.instance_index);
    }
    // a frame is drawn on its own (no TAA): last frame's place is taken as this one's
    @if(PREPASS_PIPELINE && MOTION_VECTOR_PREPASS)
    out.previous_world_position = out.world_position;
    @if(VERTEX_COLORS)
    out.color = vertex.color;
    @if(VERTEX_OUTPUT_INSTANCE_INDEX)
    out.instance_index = vertex_no_morph.instance_index;
    @if(VISIBILITY_RANGE_DITHER)
    out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(vertex_no_morph.instance_index, mesh_world_from_local[3]);
    return out;
}
