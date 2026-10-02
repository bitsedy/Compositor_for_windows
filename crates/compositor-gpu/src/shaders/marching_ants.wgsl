struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(in_vertex_index == 1u) * 4 - 1);
    let y = f32(i32(in_vertex_index == 2u) * 4 - 1);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

struct AntsUniforms {
    viewport_size: vec2<f32>,
    mask_size: vec2<f32>,
    phase: f32,
    threshold: f32,
    padding: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: AntsUniforms;
@group(0) @binding(1) var mask_tex: texture_2d<f32>;
@group(0) @binding(2) var mask_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let center = textureSample(mask_tex, mask_sampler, uv).r;
    
    let px = vec2<f32>(1.0 / uniforms.mask_size.x, 1.0 / uniforms.mask_size.y);
    let left  = textureSample(mask_tex, mask_sampler, uv - vec2<f32>(px.x, 0.0)).r;
    let right = textureSample(mask_tex, mask_sampler, uv + vec2<f32>(px.x, 0.0)).r;
    let up    = textureSample(mask_tex, mask_sampler, uv - vec2<f32>(0.0, px.y)).r;
    let down  = textureSample(mask_tex, mask_sampler, uv + vec2<f32>(0.0, px.y)).r;

    let thresh = uniforms.threshold;
    let is_inside = center >= thresh;
    let min_neighbor = min(min(left, right), min(up, down));
    let max_neighbor = max(max(left, right), max(up, down));

    let is_boundary = (is_inside && min_neighbor < thresh) || (!is_inside && max_neighbor >= thresh);

    if (!is_boundary) {
        discard;
    }

    let screen_pos = in.position.xy;
    let dash = (i32(screen_pos.x + screen_pos.y + uniforms.phase) % 8 + 8) % 8;
    if (dash < 4) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    }
}
