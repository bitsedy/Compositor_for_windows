struct NoiseParameters {
    amount: f32,
    gaussian: u32,
    monochromatic: u32,
    seed: u32,
    corner: vec2<i32>,
    _pad: vec2<i32>,
};

struct GrainParameters {
    strength: f32,
    roughness: f32,
    size: f32,
    detailSize: f32,
    origin: vec2<f32>,
    unitsPerPixel: f32,
    seed: u32,
};

struct PlaceUniforms {
    place: vec4<i32>, // minX, maxY - 1, minX - inMinX, inMaxY - outMaxY
};

fn mix32(in_x: u32) -> u32 {
    var x = in_x;
    x = x ^ (x >> 16u);
    x = x * 0x7feb352du;
    x = x ^ (x >> 15u);
    x = x * 0x846ca68bu;
    x = x ^ (x >> 16u);
    return x;
}

fn unit(key: u32) -> f32 {
    return f32(mix32(key) >> 8u) * (1.0 / 16777216.0);
}

fn lattice(ix: i32, iy: i32, seed: u32) -> f32 {
    let h = mix32(u32(ix) * 0x9E3779B1u ^ mix32(u32(iy) * 0x85EBCA77u ^ seed));
    return f32(h & 0xFFFFu) / 65535.0 + f32(h >> 16u) / 65535.0 - 1.0;
}

fn grain_field(u: f32, v: f32, scale: f32, seed: u32) -> f32 {
    let cellX = floor(u / scale);
    let cellY = floor(v / scale);
    var tx = u / scale - cellX;
    var ty = v / scale - cellY;
    tx = tx * tx * (3.0 - 2.0 * tx);
    ty = ty * ty * (3.0 - 2.0 * ty);
    let ix = i32(cellX);
    let iy = i32(cellY);
    let n00 = lattice(ix, iy, seed);
    let n10 = lattice(ix + 1, iy, seed);
    let n01 = lattice(ix, iy + 1, seed);
    let n11 = lattice(ix + 1, iy + 1, seed);
    let top = n00 + (n10 - n00) * tx;
    let bottom = n01 + (n11 - n01) * tx;
    return (top + (bottom - top) * ty) * 1.6;
}

@group(0) @binding(0) var source_noise_tex: texture_2d<f32>;
@group(0) @binding(1) var target_noise_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> p_noise: NoiseParameters;
@group(0) @binding(3) var<uniform> pl_noise: PlaceUniforms;

@compute @workgroup_size(16, 16)
fn add_noise(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dim = textureDimensions(source_noise_tex);
    if (gid.x >= dim.x || gid.y >= dim.y) { return; }

    let color = textureLoad(source_noise_tex, vec2<i32>(i32(gid.x), i32(gid.y)) + pl_noise.place.zw, 0);
    let alpha = color.a * 255.0;
    if (alpha <= 0.0) {
        textureStore(target_noise_tex, vec2<i32>(i32(gid.x), i32(gid.y)), color);
        return;
    }

    let px = u32(pl_noise.place.x + i32(gid.x) + p_noise.corner.x);
    let py = u32(pl_noise.place.y - i32(gid.y) + p_noise.corner.y);
    let base = mix32(p_noise.seed ^ mix32(px * 0x9e3779b9u ^ mix32(py * 0x85ebca6bu)));
    let spread = p_noise.amount / 100.0 * 127.5;

    var rgb = color.rgb * 255.0 / color.a;
    var result = vec3<f32>(0.0);

    for (var c = 0u; c < 3u; c = c + 1u) {
        let key = select(base + c * 0x9e3779b9u, base, p_noise.monochromatic == 1u);
        var n = 0.0;
        if (p_noise.gaussian == 1u) {
            let u1 = unit(key);
            let u2 = unit(key ^ 0x68e31da4u);
            n = sqrt(-2.0 * log(max(1.0 - u1, 1e-7))) * cos(6.2831853 * u2) * spread * (2.0 / 3.0);
        } else {
            n = (unit(key) * 2.0 - 1.0) * spread;
        }
        let value = clamp(rgb[c] + n, 0.0, 255.0);
        result[c] = value / 255.0 * color.a;
    }

    textureStore(target_noise_tex, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(result, color.a));
}

@group(0) @binding(0) var source_grain_tex: texture_2d<f32>;
@group(0) @binding(1) var target_grain_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> p_grain: GrainParameters;
@group(0) @binding(3) var<uniform> pl_grain: PlaceUniforms;

@compute @workgroup_size(16, 16)
fn add_grain(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dim = textureDimensions(source_grain_tex);
    if (gid.x >= dim.x || gid.y >= dim.y) { return; }

    let color = textureLoad(source_grain_tex, vec2<i32>(i32(gid.x), i32(gid.y)) + pl_grain.place.zw, 0);
    if (color.a <= 0.0) {
        textureStore(target_grain_tex, vec2<i32>(i32(gid.x), i32(gid.y)), color);
        return;
    }

    let u = p_grain.origin.x + (f32(pl_grain.place.x + i32(gid.x)) + 0.5) * p_grain.unitsPerPixel;
    let v = p_grain.origin.y + (f32(pl_grain.place.y - i32(gid.y)) + 0.5) * p_grain.unitsPerPixel;

    let smooth_val = grain_field(u, v, p_grain.size, p_grain.seed);
    let fine = grain_field(u, v, p_grain.detailSize, mix32(p_grain.seed ^ 0xA511E9B3u));
    let noise = smooth_val + (fine - smooth_val) * p_grain.roughness;

    let rgb = color.rgb / color.a * 255.0;
    let level = min(1.0, (0.2126 * rgb.r + 0.7152 * rgb.g + 0.0722 * rgb.b) / 255.0);
    let delta = noise * p_grain.strength * (0.4 + 2.4 * level * (1.0 - level));

    let final_rgb = clamp(rgb + vec3<f32>(delta), vec3<f32>(0.0), vec3<f32>(255.0)) / 255.0 * color.a;
    textureStore(target_grain_tex, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(final_rgb, color.a));
}
