@compute @workgroup_size(1)
fn radixSortB(@builtin(global_invocation_id) id: vec3<u32>) {
    var sum = 0u;
    for(var digit = 0u; digit < 16u; digit += 1u) {
        let tmp = sorting_global.digit_histogram[id.y][digit];
        sorting_global.digit_histogram[id.y][digit] = sum;
        sum += tmp;
    }
}
