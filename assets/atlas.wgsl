struct Instance {
    transform: mat4x4<f32>,
    uv_min: vec2<f32>,
    uv_max: vec2<f32>,
};

struct VSOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0)
var atlas_texture: texture_2d<f32>;

@group(0) @binding(1)
var atlas_sampler: sampler;

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,

    @location(1) transform0: vec4<f32>,
    @location(2) transform1: vec4<f32>,
    @location(3) transform2: vec4<f32>,
    @location(4) transform3: vec4<f32>,

    @location(5) uv_min: vec2<f32>,
    @location(6) uv_max: vec2<f32>,
) -> VSOutput {
    var out: VSOutput;

    let transform = mat4x4<f32>(
        transform0,
        transform1,
        transform2,
        transform3,
    );

    // Convert [-1.0, 1.0] -> [0, 1].
    let base_uv = position * vec2(1., -1.) / 2. + vec2(0.5);

    out.position = transform * vec4(position, 1.0, 1.0);

    out.uv = mix(
        uv_min,
        uv_max,
        base_uv,
    );

    return out;
}

@fragment
fn fs_main(
    in: VSOutput,
) -> @location(0) vec4<f32> {
    return textureSample(atlas_texture, atlas_sampler, in.uv);
}