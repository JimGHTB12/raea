import bevy_hanabi::render::vfx_common::{EffectMetadata, Spawner};
// raea: the main camera's frustum, and the effect's transform (#52)
import bevy_hanabi::render::vfx_common::{SimParams, unpack_compressed_transform};

/// Key-value pair for sorting, with optional second sort key.
struct KeyValuePair {
    /// Sorting key.
    key: u32,
    /// Secondary sorting key. Sorts value with the same primary key.
    key2: u32,
    /// Value associated with the sort key(s), generally an index to some other data.
    /// Copied as is and otherwise ignored by the sorting algorithm.
    value: u32,
}

struct SortBuffer {
    count: atomic<i32>,
    pairs: array<KeyValuePair>,
}

/// Particle buffer as an array of u32. This prevents having to specialize this shader
/// for every single particle layout.
struct RawParticleBuffer {
    data: array<u32>,
}

@group(0) @binding(0) var<storage, read_write> sort_buffer : SortBuffer;
@group(0) @binding(1) var<storage, read> particle_buffer : RawParticleBuffer;
@group(0) @binding(2) var<storage, read> indirect_index_buffer : array<u32>;
// Technically read-only, but the type contains atomic<> fields and wasm is strict about it
@group(0) @binding(3) var<storage, read_write> effect_metadatas : array<EffectMetadata>;
@group(0) @binding(4) var<storage, read> spawner : Spawner;
// raea: (#52)
@group(0) @binding(5) var<uniform> sim_params : SimParams;

// raea: the flags on the sort key's offset (mod.rs's RAEA_SORT_BY_DISTANCE and RAEA_SORT_LOCAL, #52)
const RAEA_SORT_BY_DISTANCE: u32 = 0x80000000u;
const RAEA_SORT_LOCAL: u32 = 0x40000000u;

/// Fill the sorting key-value pair buffer with data to prepare for actual sorting.
@compute @workgroup_size(64)
fn sort_fill(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let thread_index = global_invocation_id.x;
    let effect_metadata_index = spawner.effect_metadata_index;
    let count = atomicLoad(&effect_metadatas[effect_metadata_index].alive_count); // TODO - atomic not needed
    if (thread_index >= count) {
        return;
    }

    let base_particle = spawner.slab_offset;

    // Read the particle index from the ping-pong column index written this frame by vfx_update.
    // After sorting we will write back into that same column, so that the sorting effectively
    // sorted the column in-place (no ping-pong column swap here).
    let read_index = effect_metadatas[effect_metadata_index].indirect_write_index;
    let particle_index = indirect_index_buffer[(base_particle + thread_index) * 3u + read_index];

    let particle_offset = (base_particle + particle_index) * effect_metadatas[effect_metadata_index].particle_stride;
    let key_offset = particle_offset + effect_metadatas[effect_metadata_index].sort_key_offset;
    let key2_offset = particle_offset + effect_metadatas[effect_metadata_index].sort_key2_offset;

    let pair_index = atomicAdd(&sort_buffer.count, 1);
    sort_buffer.pairs[pair_index].value = particle_index;

    // raea: a blended effect's key is its particle's view depth from the main camera, as Bevy sorts its transparent
    // phase: the distance in front of the near plane (frustum[4], its normal facing in), inverted so the sort's
    // ascending order draws far first (a non-negative f32's bits order as a u32 does); AGE breaks ties, or 0 (#52)
    let raea_flags = effect_metadatas[effect_metadata_index].sort_key_offset;
    if ((raea_flags & RAEA_SORT_BY_DISTANCE) != 0u) {
        let at = particle_offset + (raea_flags & ~(RAEA_SORT_BY_DISTANCE | RAEA_SORT_LOCAL));
        var position = vec3<f32>(
            bitcast<f32>(particle_buffer.data[at]),
            bitcast<f32>(particle_buffer.data[at + 1u]),
            bitcast<f32>(particle_buffer.data[at + 2u]),
        );
        if ((raea_flags & RAEA_SORT_LOCAL) != 0u) {
            position = (unpack_compressed_transform(spawner.transform) * vec4<f32>(position, 1.0)).xyz;
        }
        let near = sim_params.frustum[4];
        sort_buffer.pairs[pair_index].key = ~bitcast<u32>(max(dot(near.xyz, position) + near.w, 0.0));
        let key2 = effect_metadatas[effect_metadata_index].sort_key2_offset;
        sort_buffer.pairs[pair_index].key2 = 0u;
        if (key2 != 0xFFFFFFFFu) {
            sort_buffer.pairs[pair_index].key2 = particle_buffer.data[key2_offset];
        }
    } else {
    sort_buffer.pairs[pair_index].key = particle_buffer.data[key_offset];
    sort_buffer.pairs[pair_index].key2 = particle_buffer.data[key2_offset];
    }
}
