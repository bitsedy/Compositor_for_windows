struct SpreadUniforms {
    width: u32,
    height: u32,
    reach: u32,
    smallest: u32,
};

struct ShiftUniforms {
    width: u32,
    height: u32,
    dx: f32,
    dy: f32,
};

struct BlurUniforms {
    width: u32,
    height: u32,
    sigma: f32,
    radius: u32,
};

struct ComposeUniforms {
    width: u32,
    height: u32,
    _pad0: u32,
    _pad1: u32,
    strokeColor: vec4<f32>,
    shadowColor: vec4<f32>,
    overlayColor: vec4<f32>,
    innerColor: vec4<f32>,
    glowColor: vec4<f32>,
    innerGlowColor: vec4<f32>,
    flags: vec4<u32>, // has_stroke, stroke_inside, has_shadow, has_inner_shadow
    more: vec4<u32>,  // has_color_overlay, has_outer_glow, has_inner_glow, unused
};

@group(0) @binding(0) var<storage, read> in_pixels: array<u32>;
@group(0) @binding(1) var<storage, read_write> out_coverage: array<f32>;
@group(0) @binding(2) var<uniform> u_spread: SpreadUniforms;

@compute @workgroup_size(16, 16)
fn effects_alpha(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= u_spread.width || gid.y >= u_spread.height) { return; }
    let index = gid.y * u_spread.width + gid.x;
    let pixel = in_pixels[index];
    let alpha = f32((pixel >> 24u) & 0xFFu) / 255.0;
    out_coverage[index] = alpha;
}

@group(0) @binding(0) var<storage, read> spread_in: array<f32>;
@group(0) @binding(1) var<storage, read_write> spread_out: array<f32>;
@group(0) @binding(2) var<uniform> u_spread2: SpreadUniforms;

@compute @workgroup_size(16, 16)
fn effects_spread_rows(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= u_spread2.width || gid.y >= u_spread2.height) { return; }
    let reach = i32(u_spread2.reach);
    let x = i32(gid.x);
    var best = select(0.0, 1.0, u_spread2.smallest == 1u);
    for (var offset = -reach; offset <= reach; offset = offset + 1) {
        let sample_x = x + offset;
        let value = select(
            0.0,
            spread_in[gid.y * u_spread2.width + u32(sample_x)],
            sample_x >= 0 && sample_x < i32(u_spread2.width)
        );
        best = select(max(best, value), min(best, value), u_spread2.smallest == 1u);
    }
    spread_out[gid.y * u_spread2.width + gid.x] = best;
}

@compute @workgroup_size(16, 16)
fn effects_spread_columns(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= u_spread2.width || gid.y >= u_spread2.height) { return; }
    let reach = i32(u_spread2.reach);
    let y = i32(gid.y);
    var best = select(0.0, 1.0, u_spread2.smallest == 1u);
    for (var offset = -reach; offset <= reach; offset = offset + 1) {
        let sample_y = y + offset;
        let value = select(
            0.0,
            spread_in[u32(sample_y) * u_spread2.width + gid.x],
            sample_y >= 0 && sample_y < i32(u_spread2.height)
        );
        best = select(max(best, value), min(best, value), u_spread2.smallest == 1u);
    }
    spread_out[gid.y * u_spread2.width + gid.x] = best;
}

@group(0) @binding(0) var<storage, read> ring_shape: array<f32>;
@group(0) @binding(1) var<storage, read> ring_moved: array<f32>;
@group(0) @binding(2) var<storage, read_write> ring_out: array<f32>;
@group(0) @binding(3) var<uniform> u_ring: SpreadUniforms;

@compute @workgroup_size(16, 16)
fn effects_ring(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= u_ring.width || gid.y >= u_ring.height) { return; }
    let index = gid.y * u_ring.width + gid.x;
    let s = ring_shape[index];
    let m = ring_moved[index];
    let diff = select(m - s, s - m, u_ring.smallest == 1u);
    ring_out[index] = clamp(diff, 0.0, 1.0);
}

@group(0) @binding(0) var<storage, read> shift_in: array<f32>;
@group(0) @binding(1) var<storage, read_write> shift_out: array<f32>;
@group(0) @binding(2) var<uniform> u_shift: ShiftUniforms;

@compute @workgroup_size(16, 16)
fn effects_shift(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= u_shift.width || gid.y >= u_shift.height) { return; }
    let sx = f32(gid.x) - u_shift.dx;
    let sy = f32(gid.y) - u_shift.dy;
    var value = 0.0;
    if (sx >= 0.0 && sy >= 0.0 && sx <= f32(u_shift.width - 1u) && sy <= f32(u_shift.height - 1u)) {
        let x0 = u32(floor(sx));
        let y0 = u32(floor(sy));
        let x1 = min(x0 + 1u, u_shift.width - 1u);
        let y1 = min(y0 + 1u, u_shift.height - 1u);
        let fx = sx - f32(x0);
        let fy = sy - f32(y0);
        let top = mix(shift_in[y0 * u_shift.width + x0], shift_in[y0 * u_shift.width + x1], fx);
        let bottom = mix(shift_in[y1 * u_shift.width + x0], shift_in[y1 * u_shift.width + x1], fx);
        value = mix(top, bottom, fy);
    }
    shift_out[gid.y * u_shift.width + gid.x] = value;
}

@group(0) @binding(0) var<storage, read> blur_in: array<f32>;
@group(0) @binding(1) var<storage, read_write> blur_out: array<f32>;
@group(0) @binding(2) var<uniform> u_blur: BlurUniforms;

@compute @workgroup_size(16, 16)
fn effects_blur_rows(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= u_blur.width || gid.y >= u_blur.height) { return; }
    let radius = i32(u_blur.radius);
    var total = 0.0;
    var weightSum = 0.0;
    for (var offset = -radius; offset <= radius; offset = offset + 1) {
        let weight = exp(-f32(offset * offset) / (2.0 * u_blur.sigma * u_blur.sigma));
        let sample_x = clamp(i32(gid.x) + offset, 0, i32(u_blur.width) - 1);
        total = total + weight * blur_in[gid.y * u_blur.width + u32(sample_x)];
        weightSum = weightSum + weight;
    }
    blur_out[gid.y * u_blur.width + gid.x] = total / max(weightSum, 1e-6);
}

@compute @workgroup_size(16, 16)
fn effects_blur_columns(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= u_blur.width || gid.y >= u_blur.height) { return; }
    let radius = i32(u_blur.radius);
    var total = 0.0;
    var weightSum = 0.0;
    for (var offset = -radius; offset <= radius; offset = offset + 1) {
        let weight = exp(-f32(offset * offset) / (2.0 * u_blur.sigma * u_blur.sigma));
        let sample_y = clamp(i32(gid.y) + offset, 0, i32(u_blur.height) - 1);
        total = total + weight * blur_in[u32(sample_y) * u_blur.width + gid.x];
        weightSum = weightSum + weight;
    }
    blur_out[gid.y * u_blur.width + gid.x] = total / max(weightSum, 1e-6);
}

@group(0) @binding(0) var<storage, read> in_shape: array<f32>;
@group(0) @binding(1) var<storage, read> in_moved: array<f32>;
@group(0) @binding(2) var<storage, read_write> inside_out: array<f32>;
@group(0) @binding(3) var<uniform> u_inside: SpreadUniforms;

@compute @workgroup_size(16, 16)
fn effects_inside(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= u_inside.width || gid.y >= u_inside.height) { return; }
    let index = gid.y * u_inside.width + gid.x;
    inside_out[index] = clamp(in_shape[index] * (1.0 - in_moved[index]), 0.0, 1.0);
}

@group(0) @binding(0) var<storage, read> comp_pixels: array<u32>;
@group(0) @binding(1) var<storage, read> comp_ring: array<f32>;
@group(0) @binding(2) var<storage, read> comp_shadow: array<f32>;
@group(0) @binding(3) var<storage, read_write> comp_result: array<u32>;
@group(0) @binding(4) var<storage, read> comp_inner: array<f32>;
@group(0) @binding(5) var<storage, read> comp_shape: array<f32>;
@group(0) @binding(6) var<storage, read> comp_glow: array<f32>;
@group(0) @binding(7) var<storage, read> comp_innerGlow: array<f32>;
@group(0) @binding(8) var<uniform> u_comp: ComposeUniforms;

@compute @workgroup_size(16, 16)
fn effects_compose(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= u_comp.width || gid.y >= u_comp.height) { return; }
    let index = gid.y * u_comp.width + gid.x;

    var color = vec3<f32>(0.0);
    var alpha = 0.0;

    // Drop shadow
    if (u_comp.flags.z == 1u) {
        let coverage = clamp(comp_shadow[index] * u_comp.shadowColor.w, 0.0, 1.0);
        color = u_comp.shadowColor.xyz * coverage;
        alpha = coverage;
    }

    // Outer glow
    if (u_comp.more.y == 1u) {
        let glowCoverage = clamp(comp_glow[index] * (1.0 - comp_shape[index]) * u_comp.glowColor.w, 0.0, 1.0);
        color = u_comp.glowColor.xyz * glowCoverage + color * (1.0 - glowCoverage);
        alpha = glowCoverage + alpha * (1.0 - glowCoverage);
    }

    // Outside stroke
    let strokeCoverage = select(0.0, clamp(comp_ring[index] * u_comp.strokeColor.w, 0.0, 1.0), u_comp.flags.x == 1u);
    if (u_comp.flags.x == 1u && u_comp.flags.y == 0u) {
        color = u_comp.strokeColor.xyz * strokeCoverage + color * (1.0 - strokeCoverage);
        alpha = strokeCoverage + alpha * (1.0 - strokeCoverage);
    }

    // Base source layer pixels (unpacked RGBA8)
    let p = comp_pixels[index];
    let src_r = f32(p & 0xFFu) / 255.0;
    let src_g = f32((p >> 8u) & 0xFFu) / 255.0;
    let src_b = f32((p >> 16u) & 0xFFu) / 255.0;
    let src_a = f32((p >> 24u) & 0xFFu) / 255.0;

    color = vec3<f32>(src_r, src_g, src_b) + color * (1.0 - src_a);
    alpha = src_a + alpha * (1.0 - src_a);

    // Color overlay
    if (u_comp.more.x == 1u) {
        let coverage = clamp(comp_shape[index] * u_comp.overlayColor.w, 0.0, 1.0);
        color = u_comp.overlayColor.xyz * coverage + color * (1.0 - coverage);
        alpha = coverage + alpha * (1.0 - coverage);
    }

    // Inner glow
    if (u_comp.more.z == 1u) {
        let coverage = clamp(comp_innerGlow[index] * u_comp.innerGlowColor.w, 0.0, 1.0);
        color = u_comp.innerGlowColor.xyz * coverage + color * (1.0 - coverage);
        alpha = coverage + alpha * (1.0 - coverage);
    }

    // Inner shadow
    if (u_comp.flags.w == 1u) {
        let coverage = clamp(comp_inner[index] * u_comp.innerColor.w, 0.0, 1.0);
        color = u_comp.innerColor.xyz * coverage + color * (1.0 - coverage);
        alpha = coverage + alpha * (1.0 - coverage);
    }

    // Inside stroke
    if (u_comp.flags.x == 1u && u_comp.flags.y == 1u) {
        color = u_comp.strokeColor.xyz * strokeCoverage + color * (1.0 - strokeCoverage);
        alpha = strokeCoverage + alpha * (1.0 - strokeCoverage);
    }

    let out_r = u32(clamp(color.x, 0.0, 1.0) * 255.0 + 0.5);
    let out_g = u32(clamp(color.y, 0.0, 1.0) * 255.0 + 0.5);
    let out_b = u32(clamp(color.z, 0.0, 1.0) * 255.0 + 0.5);
    let out_a = u32(clamp(alpha, 0.0, 1.0) * 255.0 + 0.5);

    comp_result[index] = out_r | (out_g << 8u) | (out_b << 16u) | (out_a << 24u);
}
