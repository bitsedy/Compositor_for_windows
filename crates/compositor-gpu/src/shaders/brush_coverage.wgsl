struct BrushUniforms {
    mapping: vec4<f32>,
    geometry: vec4<f32>, // tile origin x, y, radius, hardness
    canvas: vec4<f32>,   // canvas width, canvas height, antialias, spacing
    counts: vec4<u32>,   // tile width, tile height, settled segments, total segments
};

@group(0) @binding(0) var<storage, read_write> permanent: array<f32>;
@group(0) @binding(1) var<storage, read_write> preview: array<u32>;
@group(0) @binding(2) var<uniform> u: BrushUniforms;
@group(0) @binding(3) var<storage, read> segments: array<vec4<f32>>;

fn tipDensity(distanceSquared: f32) -> f32 {
    let dist = sqrt(distanceSquared);
    let r = u.geometry.z;
    let t = clamp((dist / r - u.geometry.w) / (1.0 - u.geometry.w), 0.0, 1.0);
    let cov = max(0.0, (exp(-2.5 * t * t) - exp(-2.5)) / (1.0 - exp(-2.5)));
    return -log(max(1.0 - cov, 0.001));
}

fn segmentDensity(p: vec2<f32>, segment: vec4<f32>) -> f32 {
    let v = segment.zw - segment.xy;
    let len = length(v);
    if (len < 1e-6) {
        let d = p - segment.xy;
        return tipDensity(dot(d, d));
    }
    let dir = v / len;
    let proj = dot(p - segment.xy, dir);
    let perp = p - segment.xy - proj * dir;
    let perpSq = dot(perp, perp);
    let rSq = u.geometry.z * u.geometry.z;
    if (perpSq >= rSq) { return 0.0; }
    let reach = sqrt(rSq - perpSq);
    let lo = max(0.0, proj - reach);
    let hi = min(len, proj + reach);
    if (hi <= lo) { return 0.0; }
    let mid = (lo + hi) * 0.5;
    let halfLen = (hi - lo) * 0.5;

    let a0 = mid - halfLen * 0.18343464 - proj;
    let b0 = mid + halfLen * 0.18343464 - proj;
    let a1 = mid - halfLen * 0.5255324 - proj;
    let b1 = mid + halfLen * 0.5255324 - proj;
    let a2 = mid - halfLen * 0.7966665 - proj;
    let b2 = mid + halfLen * 0.7966665 - proj;
    let a3 = mid - halfLen * 0.96028984 - proj;
    let b3 = mid + halfLen * 0.96028984 - proj;

    let integral = 0.3626838 * (tipDensity(perpSq + a0 * a0) + tipDensity(perpSq + b0 * b0))
                 + 0.31370664 * (tipDensity(perpSq + a1 * a1) + tipDensity(perpSq + b1 * b1))
                 + 0.22238103 * (tipDensity(perpSq + a2 * a2) + tipDensity(perpSq + b2 * b2))
                 + 0.101228536 * (tipDensity(perpSq + a3 * a3) + tipDensity(perpSq + b3 * b3));

    return integral * halfLen / u.canvas.w;
}

@compute @workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = gid.x;
    let y = gid.y;
    let tileW = u.counts.x;
    let tileH = u.counts.y;
    if (x >= tileW || y >= tileH) { return; }
    let index = y * tileW + x;

    let local = vec2<f32>(f32(x) + 0.5, f32(y) + 0.5);
    let p = u.geometry.xy + local;

    if (p.x < 0.0 || p.y < 0.0 || p.x >= u.canvas.x || p.y >= u.canvas.y) {
        return;
    }

    var val = permanent[index];
    var tail = 0.0;
    let settled_count = u.counts.z;
    let total_count = u.counts.w;

    for (var i = 0u; i < settled_count; i = i + 1u) {
        val = val + segmentDensity(p, segments[i]);
    }
    for (var i = settled_count; i < total_count; i = i + 1u) {
        tail = tail + segmentDensity(p, segments[i]);
    }

    permanent[index] = min(val, 20.0);
    let cov = 1.0 - exp(-min(val + tail, 20.0));
    preview[index] = u32(clamp(cov * 255.0, 0.0, 255.0));
}
