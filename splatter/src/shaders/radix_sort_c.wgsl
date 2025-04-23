@compute @workgroup_size(256)
fn radixSortC(@builtin(global_invocation_id) id: vec3<u32>) {
    let entry = sorting[id.x];
    let digit = (entry.key >> (sorting_global.digit * 4u)) & 15u;
    let offset = sorting_global.digit_histogram[sorting_global.digit][digit];
    sorting[atomicAdd(&sorting_global.digit_histogram[sorting_global.digit][digit], 1u) + offset] = entry;
}
