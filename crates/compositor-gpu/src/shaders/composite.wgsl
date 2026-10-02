struct BlendUniforms {
    opacity: f32,
    blend_mode: u32,
    has_mask: u32,
    _pad: u32,
    transform: mat3x3<f32>, // tile pixel -> layer pixel affine transform
};

@group(0) @binding(0) var dest_tex: texture_2d<f32>;
@group(0) @binding(1) var src_tex: texture_2d<f32>;
@group(0) @binding(2) var mask_tex: texture_2d<f32>;
@group(0) @binding(3) var out_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(4) var<uniform> u_blend: BlendUniforms;

fn blend_channels(cb: f32, cs: f32, mode: u32) -> f32 {
    switch (mode) {
        case 0u: { // Normal
            return cs;
        }
        case 1u: { // Multiply
            return cb * cs;
        }
        case 2u: { // Screen
            return cb + cs - cb * cs;
        }
        case 3u: { // Overlay
            if (cb <= 0.5) {
                return 2.0 * cb * cs;
            } else {
                return 1.0 - 2.0 * (1.0 - cb) * (1.0 - cs);
            }
        }
        case 4u: { // Darken
            return min(cb, cs);
        }
        case 5u: { // Lighten
            return max(cb, cs);
        }
        case 6u: { // ColorDodge
            if (cb <= 0.0) { return 0.0; }
            if (cs >= 1.0) { return 1.0; }
            return min(1.0, cb / (1.0 - cs));
        }
        case 7u: { // ColorBurn
            if (cb >= 1.0) { return 1.0; }
            if (cs <= 0.0) { return 0.0; }
            return 1.0 - min(1.0, (1.0 - cb) / cs);
        }
        case 8u: { // HardLight
            if (cs <= 0.5) {
                return 2.0 * cb * cs;
            } else {
                return 1.0 - 2.0 * (1.0 - cb) * (1.0 - cs);
            }
        }
        case 9u: { // SoftLight
            if (cs <= 0.5) {
                return cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb);
            } else {
                let d = select(sqrt(cb), ((16.0 * cb - 12.0) * cb + 4.0) * cb, cb <= 0.25);
                return cb + (2.0 * cs - 1.0) * (d - cb);
            }
        }
        case 10u: { // Difference
            return abs(cb - cs);
        }
        case 11u: { // Exclusion
            return cb + cs - 2.0 * cb * cs;
        }
        case 12u: { // LinearDodge (Add)
            return min(1.0, cb + cs);
        }
        case 13u: { // LinearBurn
            return max(0.0, cb + cs - 1.0);
        }
        default: {
            return cs;
        }
    }
}

@compute @workgroup_size(16, 16)
fn composite_tile(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dim = textureDimensions(out_tex);
    if (gid.x >= dim.x || gid.y >= dim.y) { return; }

    let pos = vec2<i32>(i32(gid.x), i32(gid.y));
    let dst = textureLoad(dest_tex, pos, 0);

    // Map tile coord to layer coord using transform
    let layer_coord = (u_blend.transform * vec3<f32>(f32(pos.x), f32(pos.y), 1.0)).xy;
    let src_pos = vec2<i32>(i32(round(layer_coord.x)), i32(round(layer_coord.y)));

    let src_dim = textureDimensions(src_tex);
    if (src_pos.x < 0 || src_pos.y < 0 || u32(src_pos.x) >= src_dim.x || u32(src_pos.y) >= src_dim.y) {
        textureStore(out_tex, pos, dst);
        return;
    }

    let src = textureLoad(src_tex, src_pos, 0);
    var mask_val = 1.0;
    if (u_blend.has_mask == 1u) {
        mask_val = textureLoad(mask_tex, src_pos, 0).r;
    }

    let effective_alpha = src.a * u_blend.opacity * mask_val;
    if (effective_alpha <= 0.0) {
        textureStore(out_tex, pos, dst);
        return;
    }

    let cb = select(vec3<f32>(0.0), dst.rgb / max(dst.a, 1e-6), dst.a > 0.0);
    let cs = select(vec3<f32>(0.0), src.rgb / max(src.a, 1e-6), src.a > 0.0);

    let b_rgb = vec3<f32>(
        blend_channels(cb.r, cs.r, u_blend.blend_mode),
        blend_channels(cb.g, cs.g, u_blend.blend_mode),
        blend_channels(cb.b, cs.b, u_blend.blend_mode)
    );

    // Porter-Duff source-over composite with premultiplied alpha
    let out_a = effective_alpha + dst.a * (1.0 - effective_alpha);
    let out_rgb = (b_rgb * effective_alpha + dst.rgb * (1.0 - effective_alpha));

    textureStore(out_tex, pos, vec4<f32>(out_rgb, out_a));
}
