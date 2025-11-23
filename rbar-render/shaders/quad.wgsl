
struct VertexIn {
    // 1. The small quad corner position (-0.5 to 0.5)
    @location(0) pos: vec2<f32>,

    // 2. Widget's screen position (in pixels)
    @location(1) instance_pos: vec2<f32>,

    // 3. Widget's dimensions (width/height in pixels)
    @location(2) size: vec2<f32>,

    // 4. Widget's color
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) widget_color: vec4<f32>,
};

struct Globals {
    resolution: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> u_globals: Globals;

@vertex
fn vs_main(in: VertexIn) -> VertexOutput {
    var out: VertexOutput;

    // 1. SCALING: Scale the quad's corner position by the widget's size.
    // e.g., corner (0.5, 0.5) * size (100, 50) = (50, 25)
    let scaled_pos = in.pos * in.size;

    // 2. TRANSLATION: Move the scaled corner to the widget's screen position.
    let screen_pos_px = scaled_pos + in.instance_pos;

    // 3. NORMALIZATION: Convert screen pixels to Normalized Device Coordinates (NDC: -1.0 to 1.0)
    let clip_pos_xy = (screen_pos_px / u_globals.resolution) * 2.0 - 1.0;

    // 4. OUTPUT: Set final clip position (Y-axis typically needs to be flipped for graphics APIs)
    out.clip_position = vec4<f32>(clip_pos_xy.x, clip_pos_xy.y * -1.0, 0.0, 1.0);

    // Pass data for fragment shader
    out.uv = in.pos + 0.5;
    out.widget_color = in.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.widget_color;
}
