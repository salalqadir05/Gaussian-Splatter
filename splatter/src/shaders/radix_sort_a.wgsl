// Constants (adjust values based on your needs)
const RADIX_BASE: u32 = 16u;
const RADIX_DIGIT_PLACES: u32 = 4u;
const MAX_TILE_COUNT_C: u32 = 1024u;
const ENTRIES_PER_INVOCATION_A: u32 = 256u;

// Structs
struct Uniforms {
    camera_matrix: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    view_projection_matrix: mat4x4<f32>,
    view_size: vec2<f32>,
    image_size: vec2<u32>,
    frustum_culling_tolerance: f32,
    ellipse_size_bias: f32,
    ellipse_margin: f32,
    splat_scale: f32,
}
struct DrawIndirect {
    vertex_count: u32,
    instance_count: atomic<u32>,
    base_vertex: u32,
    base_instance: u32,
}
struct SortingGlobal {
    status_counters: array<array<atomic<u32>, RADIX_BASE>, MAX_TILE_COUNT_C>,
    digit_histogram: array<array<atomic<u32>, RADIX_BASE>, RADIX_DIGIT_PLACES>,
    draw_indirect: DrawIndirect,
    assignment_counter: atomic<u32>,
}
struct Entry {
    key: u32,
    value: u32,
}
struct Splat {
    rotation: vec4<f32>,
    center: vec3<f32>,
    paddingA: f32,
    scale: vec3<f32>,
    alpha: f32,
    colorSH: array<f32, 48>,
}
struct SortingSharedA {
    digit_histogram: array<array<atomic<u32>, RADIX_BASE>, RADIX_DIGIT_PLACES>,
}

// Bindings
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<uniform> sorting_pass_index: u32;
@group(0) @binding(2) var<storage, read_write> sorting: SortingGlobal;
@group(0) @binding(3) var<storage, read_write> input_entries: array<Entry>;
@group(0) @binding(4) var<storage, read_write> output_entries: array<Entry>;
@group(0) @binding(5) var<storage, read> sorted_entries: array<Entry>;
@group(0) @binding(6) var<storage> splats: array<Splat>;

// Helper functions
fn screenToClipSpace(screen_space_pos: vec2<f32>) -> vec2<f32> {
    var result = ((screen_space_pos.xy / vec2<f32>(uniforms.image_size)) - vec2<f32>(0.5));
    return vec2<f32>(2.0 * result.x, -2.0 * result.y);
}

fn clipToScreenSpace(clip_space_pos: vec2<f32>) -> vec2<f32> {
    var result = vec2<f32>(0.5 * clip_space_pos.x, -0.5 * clip_space_pos.y) + vec2<f32>(0.5);
    return result * vec2<f32>(uniforms.image_size);
}

fn worldToClipSpace(world_pos: vec3<f32>) -> vec4<f32> {
    var homogenous_pos = uniforms.view_projection_matrix * vec4<f32>(world_pos, 1.0);
    return vec4<f32>(homogenous_pos.xyz, 1.0) / (homogenous_pos.w + 0.0000001);
}

fn isInFrustum(clip_space_pos: vec3<f32>) -> bool {
    return abs(clip_space_pos.x) < uniforms.frustum_culling_tolerance && abs(clip_space_pos.y) < uniforms.frustum_culling_tolerance && abs(clip_space_pos.z - 0.5) < 0.5;
}

fn quatToMat(p: vec4<f32>) -> mat3x3<f32> {
    var q = p * sqrt(2.0);
    var yy = q.y * q.y;
    var yz = q.y * q.z;
    var yw = q.y * q.w;
    var yx = q.y * q.x;
    var zz = q.z * q.z;
    var zw = q.z * q.w;
    var zx = q.z * q.x;
    var ww = q.w * q.w;
    var wx = q.w * q.x;
    return mat3x3<f32>(
        1.0 - zz - ww, yz + wx, yw - zx,
        yz - wx, 1.0 - yy - ww, zw + yx,
        yw + zx, zw - yx, 1.0 - yy - zz,
    );
}

// Workgroup-shared memory
var<workgroup> sorting_shared_a: SortingSharedA;

// Radix Sort A shader
@compute @workgroup_size(RADIX_BASE, RADIX_DIGIT_PLACES)
fn radixSortA(
    @builtin(local_invocation_id) gl_LocalInvocationID: vec3<u32>,
    @builtin(global_invocation_id) gl_GlobalInvocationID: vec3<u32>,
) {
    sorting_shared_a.digit_histogram[gl_LocalInvocationID.y][gl_LocalInvocationID.x] = 0u;
    workgroupBarrier();

    let thread_index = gl_GlobalInvocationID.x * RADIX_DIGIT_PLACES + gl_GlobalInvocationID.y;
    let start_entry_index = thread_index * ENTRIES_PER_INVOCATION_A;
    let end_entry_index = start_entry_index + ENTRIES_PER_INVOCATION_A;
    for(var entry_index = start_entry_index; entry_index < end_entry_index; entry_index += 1u) {
        if(entry_index >= arrayLength(&splats)) {
            continue;
        }
        var key: u32 = 0xFFFFFFFFu; // Stream compaction for frustum culling
        let clip_space_pos = worldToClipSpace(splats[entry_index].center);
        if(isInFrustum(clip_space_pos.xyz)) {
            key = u32(clip_space_pos.z * 0xFFFF.0) << 16u;
            key |= u32((clip_space_pos.x * 0.5 + 0.5) * 0xFF.0) << 8u;
            key |= u32((clip_space_pos.y * 0.5 + 0.5) * 0xFF.0);
        }
        output_entries[entry_index].key = key;
        output_entries[entry_index].value = entry_index;
        for(var shift = 0u; shift < RADIX_DIGIT_PLACES; shift += 1u) {
            let digit = (key >> (shift * RADIX_BITS_PER_DIGIT)) & (RADIX_BASE - 1u);
            atomicAdd(&sorting_shared_a.digit_histogram[shift][digit], 1u);
        }
    }
    workgroupBarrier();

    atomicAdd(&sorting.digit_histogram[gl_LocalInvocationID.y][gl_LocalInvocationID.x], sorting_shared_a.digit_histogram[gl_LocalInvocationID.y][gl_LocalInvocationID.x]);
}
