struct VertexIn {
    @location(0) pos: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>, // 0.0 to 1.0
};

@vertex
fn vs_main(
    in: VertexIn
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.pos * 2.0, 0.0, 1.0);

    out.uv = in.pos + 0.5;
    return out;
}

struct Uniforms {
    size: vec2<f32>,
    radius: f32,
    padding: f32,
    color: vec4<f32>,
};

fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + r;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0, 0.0))) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = vec4<f32>(1.0, 0.0, 0.0, 1.0);

    let radius = 10.0;
    let resolution = vec2<f32>(1920.0, 30.0);

    let pos_px = (in.uv - 0.5) * resolution;
    let half_size = resolution / 2.0;

    let dist = sd_rounded_box(pos_px, half_size, radius);

    let alpha = 1.0 - smoothstep(0.0, 1.5, dist);

    return vec4<f32>(color.rgb, color.a * alpha);
}
