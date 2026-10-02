struct DabUniforms {
    center: vec2<i32>,
    radius: i32,
    _pad0: i32,
    size: vec2<i32>,
    origin: vec2<i32>,
    area: vec2<i32>,
    inverseRadius: f32,
    hardness: f32,
    keep: f32,
    _pad1: f32,
    mov: vec2<f32>,
};

fn weight(u: f32, hardness: f32) -> f32 {
    if (u >= 1.0) { return 0.0; }
    if (u <= hardness) { return 1.0; }
    let t = (1.0 - u) / (1.0 - hardness);
    return t * t * (3.0 - 2.0 * t);
}

@group(0) @binding(0) var canvas_tex: texture_2d<f32>;
@group(0) @binding(1) var carried_tex_write: texture_storage_2d<rgba32float, write>;
@group(0) @binding(2) var<uniform> d: DabUniforms;

@compute @workgroup_size(16, 16)
fn warp_pick_up(@builtin(global_invocation_id) gid: vec3<u32>) {
    let side = 2 * d.radius + 1;
    if (i32(gid.x) >= side || i32(gid.y) >= side) { return; }
    let p = d.center + vec2<i32>(i32(gid.x), i32(gid.y)) - vec2<i32>(d.radius);
    let inside = p.x >= 0 && p.y >= 0 && p.x < d.size.x && p.y < d.size.y;
    let color = select(
        vec4<f32>(0.0),
        textureLoad(canvas_tex, vec2<i32>(p.x, p.y), 0) * 255.0,
        inside
    );
    textureStore(carried_tex_write, vec2<i32>(i32(gid.x), i32(gid.y)), color);
}

@group(0) @binding(0) var carried_read: texture_2d<f32>;
@group(0) @binding(1) var carried_write: texture_storage_2d<rgba32float, write>;
@group(0) @binding(2) var canvas_in: texture_2d<f32>;
@group(0) @binding(3) var canvas_out: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(4) var<uniform> d_smudge: DabUniforms;

@compute @workgroup_size(16, 16)
fn warp_smudge(@builtin(global_invocation_id) gid: vec3<u32>) {
    let side = 2 * d_smudge.radius + 1;
    if (i32(gid.x) >= side || i32(gid.y) >= side) { return; }
    let offset = vec2<i32>(i32(gid.x), i32(gid.y)) - vec2<i32>(d_smudge.radius);
    let p = d_smudge.center + offset;
    if (p.x < 0 || p.y < 0 || p.x >= d_smudge.size.x || p.y >= d_smudge.size.y) { return; }

    let dist = sqrt(f32(offset.x * offset.x + offset.y * offset.y));
    let w = weight(dist * d_smudge.inverseRadius, d_smudge.hardness);
    if (w <= 0.0) { return; }

    let under = textureLoad(canvas_in, p, 0) * 255.0;
    let held = textureLoad(carried_read, vec2<i32>(i32(gid.x), i32(gid.y)), 0);

    let painted = under + (held - under) * w * d_smudge.keep;
    let final_color = clamp(round(painted), vec4<f32>(0.0), vec4<f32>(255.0)) / 255.0;

    textureStore(canvas_out, p, final_color);
    textureStore(carried_write, vec2<i32>(i32(gid.x), i32(gid.y)), painted);
}

@group(0) @binding(0) var offsets_write: texture_storage_2d<rg32float, write>;
@group(0) @binding(1) var before_tex: texture_2d<f32>;
@group(0) @binding(2) var original_tex: texture_2d<f32>;
@group(0) @binding(3) var canvas_warp_out: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(4) var<uniform> d_push: DabUniforms;

@compute @workgroup_size(16, 16)
fn warp_push(@builtin(global_invocation_id) gid: vec3<u32>) {
    let side = 2 * d_push.radius + 1;
    if (i32(gid.x) >= side || i32(gid.y) >= side) { return; }
    let offset = vec2<i32>(i32(gid.x), i32(gid.y)) - vec2<i32>(d_push.radius);
    let p = d_push.center + offset;
    let last = d_push.origin + d_push.area - vec2<i32>(1);
    if (p.x < d_push.origin.x || p.y < d_push.origin.y || p.x > last.x || p.y > last.y) { return; }

    let dist = sqrt(f32(offset.x * offset.x + offset.y * offset.y));
    let w = weight(dist * d_push.inverseRadius, d_push.hardness);
    if (w <= 0.0) { return; }

    let sx = min(f32(d_push.area.x - 1), max(0.0, f32(p.x - d_push.origin.x) - d_push.mov.x * w));
    let sy = min(f32(d_push.area.y - 1), max(0.0, f32(p.y - d_push.origin.y) - d_push.mov.y * w));
    let ix = min(d_push.area.x - 2, i32(sx));
    let iy = min(d_push.area.y - 2, i32(sy));
    if (ix < 0 || iy < 0) { return; }

    let fx = sx - f32(ix);
    let fy = sy - f32(iy);

    let o00 = textureLoad(before_tex, vec2<i32>(ix, iy), 0).xy;
    let o10 = textureLoad(before_tex, vec2<i32>(ix + 1, iy), 0).xy;
    let o01 = textureLoad(before_tex, vec2<i32>(ix, iy + 1), 0).xy;
    let o11 = textureLoad(before_tex, vec2<i32>(ix + 1, iy + 1), 0).xy;

    let moved = mix(mix(o00, o10, fx), mix(o01, o11, fx), fy) - d_push.mov * w;
    textureStore(offsets_write, p, vec4<f32>(moved, 0.0, 0.0));

    let source = clamp(vec2<f32>(f32(p.x), f32(p.y)) + moved, vec2<f32>(0.0), vec2<f32>(d_push.size - vec2<i32>(1)));
    let i_coord = min(vec2<i32>(source), d_push.size - vec2<i32>(2));
    let f_coord = source - vec2<f32>(i_coord);

    let c00 = textureLoad(original_tex, i_coord, 0);
    let c10 = textureLoad(original_tex, i_coord + vec2<i32>(1, 0), 0);
    let c01 = textureLoad(original_tex, i_coord + vec2<i32>(0, 1), 0);
    let c11 = textureLoad(original_tex, i_coord + vec2<i32>(1, 1), 0);

    let color = mix(mix(c00, c10, f_coord.x), mix(c01, c11, f_coord.x), f_coord.y);
    textureStore(canvas_warp_out, p, color);
}
