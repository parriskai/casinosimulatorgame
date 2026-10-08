struct GpuPoint {
    position: vec4<f32>,
};

struct VertexInput {
    // Shared vertex.
    // 0 = start, 1 = end.
    @location(0) t: f32,

    // Per-instance point indices.
    @location(1) start: u32,
    @location(2) end: u32,

    @location(3) start_color: vec4<f32>,
    @location(4) end_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@group(0) @binding(0)
var<storage, read> points: array<GpuPoint>;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let start = points[input.start].position;
    let end = points[input.end].position;

    output.position = mix(start, end, input.t);

    output.color = mix(
        input.start_color,
        input.end_color,
        input.t
    );

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}