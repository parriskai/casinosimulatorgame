struct Globals {
    world_to_cvv: mat4x4<f32>,
};

// struct Instance {
//     position: vec2<f32>,
//     size: vec2<f32>,
//     uv_index: u32,
// };

@group(0) @binding(0)
var atlas: texture_2d<f32>;

@group(0) @binding(1)
var atlas_sampler: sampler;

@group(0) @binding(2)
var<uniform> globals: Globals;

@group(0) @binding(3)
var<storage, read> uv_table: array<vec4<f32>>;


struct VertexInput {
    @location(0) quad_position: vec2<f32>,

    @location(1) position: vec2<i32>,
    @location(2) uv_index: u32,
};


struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};


@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let world_position = vec2<f32>(input.position) + input.quad_position;

    output.position = 
        globals.world_to_cvv *
        vec4<f32>(world_position, 0.0, 1.0);

    let uv_rect = uv_table[input.uv_index];

    let uv_min = uv_rect.xy;
    let uv_max = uv_rect.zw;

    output.uv =
        mix(uv_min, uv_max, input.quad_position);

    return output;
}


@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(
         atlas,
         atlas_sampler,
         input.uv
    );
}